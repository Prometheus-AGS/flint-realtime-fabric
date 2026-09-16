use aes_gcm::aead::{Aead as _, KeyInit as _, Payload as AeadPayload};
use aes_gcm::{Aes256Gcm, Nonce};
use frf_domain::{EntityTypeSelector, TenantId};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::entity_type_watch::WatchCheckpoint;

const VERSION: u32 = 1;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckpointPayload {
    version: u32,
    generation: u64,
    subject: String,
    tenant_id: TenantId,
    entity_type: EntityTypeSelector,
    source_epoch: String,
    last_offset: Option<u64>,
}

pub(crate) struct CheckpointScope<'a> {
    pub subject: &'a str,
    pub tenant_id: TenantId,
    pub entity_type: &'a EntityTypeSelector,
    pub source_epoch: &'a str,
}

#[derive(Clone)]
pub(crate) struct CheckpointCodec {
    key: [u8; KEY_LEN],
    generation: u64,
}

impl CheckpointCodec {
    pub(crate) fn new(key: &[u8], generation: u64) -> Result<Self, String> {
        if key.len() < KEY_LEN {
            return Err("entity watch checkpoint key must contain at least 32 bytes".to_owned());
        }
        let digest = Sha256::digest([b"frf-watch-checkpoint-v1".as_slice(), key].concat());
        let mut derived = [0_u8; KEY_LEN];
        derived.copy_from_slice(&digest);
        Ok(Self {
            key: derived,
            generation,
        })
    }

    pub(crate) fn issue(
        &self,
        scope: &CheckpointScope<'_>,
        last_offset: Option<u64>,
    ) -> Result<WatchCheckpoint, String> {
        let payload = CheckpointPayload {
            version: VERSION,
            generation: self.generation,
            subject: scope.subject.to_owned(),
            tenant_id: scope.tenant_id,
            entity_type: scope.entity_type.clone(),
            source_epoch: scope.source_epoch.to_owned(),
            last_offset,
        };
        let plaintext = serde_json::to_vec(&payload).map_err(|error| error.to_string())?;
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|error| format!("checkpoint encryption key: {error}"))?;
        let mut nonce = [0_u8; NONCE_LEN];
        getrandom::fill(&mut nonce).map_err(|error| format!("checkpoint nonce: {error}"))?;
        let aad = checkpoint_aad(VERSION, self.generation);
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                AeadPayload {
                    msg: &plaintext,
                    aad: &aad,
                },
            )
            .map_err(|error| format!("checkpoint encryption: {error}"))?;
        let mut token = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        token.extend_from_slice(&nonce);
        token.extend_from_slice(&ciphertext);
        Ok(WatchCheckpoint {
            version: VERSION,
            token,
            generation: self.generation,
        })
    }

    pub(crate) fn validate(
        &self,
        checkpoint: &WatchCheckpoint,
        scope: &CheckpointScope<'_>,
    ) -> Result<Option<u64>, CheckpointValidation> {
        if checkpoint.version != VERSION {
            return Err(CheckpointValidation::Unsupported);
        }
        if checkpoint.generation != self.generation {
            return Err(CheckpointValidation::HistoryExpired);
        }
        if checkpoint.token.len() <= NONCE_LEN + TAG_LEN {
            return Err(CheckpointValidation::Invalid);
        }
        let (nonce, ciphertext) = checkpoint.token.split_at(NONCE_LEN);
        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|_| CheckpointValidation::Invalid)?;
        let aad = checkpoint_aad(checkpoint.version, checkpoint.generation);
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(nonce),
                AeadPayload {
                    msg: ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| CheckpointValidation::Invalid)?;
        let payload: CheckpointPayload =
            serde_json::from_slice(&plaintext).map_err(|_| CheckpointValidation::Invalid)?;
        if payload.version != VERSION {
            return Err(CheckpointValidation::Unsupported);
        }
        if payload.generation != self.generation {
            return Err(CheckpointValidation::HistoryExpired);
        }
        if payload.subject != scope.subject
            || payload.tenant_id != scope.tenant_id
            || payload.entity_type != *scope.entity_type
        {
            return Err(CheckpointValidation::ScopeMismatch);
        }
        if payload.source_epoch != scope.source_epoch {
            return Err(CheckpointValidation::SourceEpochChanged);
        }
        Ok(payload.last_offset)
    }
}

fn checkpoint_aad(version: u32, generation: u64) -> [u8; 12] {
    let mut aad = [0_u8; 12];
    aad[..4].copy_from_slice(&version.to_be_bytes());
    aad[4..].copy_from_slice(&generation.to_be_bytes());
    aad
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckpointValidation {
    HistoryExpired,
    SourceEpochChanged,
    ScopeMismatch,
    Unsupported,
    Invalid,
}

#[cfg(test)]
mod tests {
    use frf_domain::{EntityTypeSelector, TenantId};

    use super::{CheckpointCodec, CheckpointScope, CheckpointValidation};

    fn selector(name: &str) -> EntityTypeSelector {
        EntityTypeSelector {
            schema: "public".to_owned(),
            name: name.to_owned(),
            projection: "default".to_owned(),
        }
    }

    #[test]
    fn checkpoint_integrity_scope_epoch_and_generation_fail_closed() {
        let tenant = TenantId::from_uuid(uuid::Uuid::from_u128(1));
        let entity_type = selector("orders");
        let codec = CheckpointCodec::new(&[7; 32], 3).unwrap_or_else(|error| panic!("{error}"));
        let scope = CheckpointScope {
            subject: "reader",
            tenant_id: tenant,
            entity_type: &entity_type,
            source_epoch: "epoch-a",
        };
        let checkpoint = codec
            .issue(&scope, Some(41))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(codec.validate(&checkpoint, &scope), Ok(Some(41)));
        assert!(
            !checkpoint
                .token
                .windows(scope.subject.len())
                .any(|window| window == scope.subject.as_bytes())
        );
        assert!(
            !checkpoint
                .token
                .windows(scope.source_epoch.len())
                .any(|window| window == scope.source_epoch.as_bytes())
        );
        let retry = codec
            .issue(&scope, Some(41))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_ne!(checkpoint.token, retry.token);

        let mut tampered = checkpoint.clone();
        tampered.token[0] ^= 1;
        assert_eq!(
            codec.validate(&tampered, &scope),
            Err(CheckpointValidation::Invalid)
        );
        let other_type = selector("notes");
        let wrong_scope = CheckpointScope {
            subject: "reader",
            tenant_id: tenant,
            entity_type: &other_type,
            source_epoch: "epoch-a",
        };
        assert_eq!(
            codec.validate(&checkpoint, &wrong_scope),
            Err(CheckpointValidation::ScopeMismatch)
        );
        let wrong_epoch = CheckpointScope {
            subject: "reader",
            tenant_id: tenant,
            entity_type: &entity_type,
            source_epoch: "epoch-b",
        };
        assert_eq!(
            codec.validate(&checkpoint, &wrong_epoch),
            Err(CheckpointValidation::SourceEpochChanged)
        );
        let next_generation =
            CheckpointCodec::new(&[7; 32], 4).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            next_generation.validate(&checkpoint, &scope),
            Err(CheckpointValidation::HistoryExpired)
        );
    }
}
