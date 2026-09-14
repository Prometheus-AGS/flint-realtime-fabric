//! Settlement of process-local Electric continuation bindings.

use std::sync::{Arc, Mutex, MutexGuard};

use super::lease::StreamOutcome;
use super::{HandleBinding, HandleBindings};

pub(super) struct ContinuationSettlement {
    bindings: Arc<Mutex<HandleBindings>>,
    previous_handle: Option<String>,
    response_handle: Option<String>,
    binding: HandleBinding,
    now_epoch_seconds: u64,
}

impl ContinuationSettlement {
    pub(super) fn new(
        bindings: Arc<Mutex<HandleBindings>>,
        previous_handle: Option<String>,
        response_handle: Option<String>,
        binding: HandleBinding,
        now_epoch_seconds: u64,
    ) -> Self {
        Self {
            bindings,
            previous_handle,
            response_handle,
            binding,
            now_epoch_seconds,
        }
    }

    pub(super) fn settle(&self, outcome: StreamOutcome) {
        let mut bindings = lock_bindings(&self.bindings);
        prune_expired(&mut bindings, self.now_epoch_seconds);
        match outcome {
            StreamOutcome::Completed => self.commit(&mut bindings),
            StreamOutcome::Cancelled => self.release_previous(&mut bindings),
        }
    }

    fn commit(&self, bindings: &mut HandleBindings) {
        let Some(response_handle) = self.response_handle.as_ref() else {
            return;
        };
        if self.previous_handle.as_ref() != Some(response_handle) {
            self.release_previous(bindings);
        }
        let response_bindings = bindings.entry(response_handle.clone()).or_default();
        response_bindings.retain(|existing| !existing.same_authority(&self.binding));
        response_bindings.push(self.binding.clone());
    }

    fn release_previous(&self, bindings: &mut HandleBindings) {
        let Some(previous_handle) = self.previous_handle.as_ref() else {
            return;
        };
        let remove_handle = if let Some(previous_bindings) = bindings.get_mut(previous_handle) {
            previous_bindings.retain(|existing| !existing.same_authority(&self.binding));
            previous_bindings.is_empty()
        } else {
            false
        };
        if remove_handle {
            bindings.remove(previous_handle);
        }
    }
}

pub(super) fn lock_bindings(bindings: &Mutex<HandleBindings>) -> MutexGuard<'_, HandleBindings> {
    match bindings.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn prune_expired(bindings: &mut HandleBindings, now_epoch_seconds: u64) {
    bindings.retain(|_, handle_bindings| {
        handle_bindings.retain(|binding| binding.expires_at > now_epoch_seconds);
        !handle_bindings.is_empty()
    });
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use frf_domain::{SessionId, TenantId};

    use super::*;

    fn binding(subject: &str) -> HandleBinding {
        HandleBinding {
            tenant_id: TenantId::from_uuid(uuid::Uuid::from_u128(1)),
            subject: subject.to_owned(),
            originating_session_id: Some(SessionId::from_uuid(uuid::Uuid::from_u128(2))),
            authorization_revision: Some("membership:1".to_owned()),
            projection_revision: Some(1),
            projection_ids: vec!["cases".to_owned()],
            shape: "cases".to_owned(),
            expires_at: 2_000,
        }
    }

    #[test]
    fn normal_completion_replaces_the_previous_handle() {
        let owner = binding("subject-1");
        let bindings = Arc::new(Mutex::new(HashMap::from([(
            "old".to_owned(),
            vec![owner.clone()],
        )])));
        let settlement = ContinuationSettlement::new(
            Arc::clone(&bindings),
            Some("old".to_owned()),
            Some("new".to_owned()),
            owner.clone(),
            1_000,
        );

        settlement.settle(StreamOutcome::Completed);

        let bindings = lock_bindings(&bindings);
        assert!(!bindings.contains_key("old"));
        assert_eq!(bindings.get("new"), Some(&vec![owner]));
    }

    #[test]
    fn cancellation_releases_only_the_resuming_authority() {
        let owner = binding("subject-1");
        let other = binding("subject-2");
        let bindings = Arc::new(Mutex::new(HashMap::from([(
            "shared".to_owned(),
            vec![owner.clone(), other.clone()],
        )])));
        let settlement = ContinuationSettlement::new(
            Arc::clone(&bindings),
            Some("shared".to_owned()),
            Some("replacement".to_owned()),
            owner,
            1_000,
        );

        settlement.settle(StreamOutcome::Cancelled);

        let bindings = lock_bindings(&bindings);
        assert_eq!(bindings.get("shared"), Some(&vec![other]));
        assert!(!bindings.contains_key("replacement"));
    }

    #[test]
    fn completed_not_modified_response_preserves_the_previous_handle() {
        let owner = binding("subject-1");
        let bindings = Arc::new(Mutex::new(HashMap::from([(
            "current".to_owned(),
            vec![owner.clone()],
        )])));
        let settlement = ContinuationSettlement::new(
            Arc::clone(&bindings),
            Some("current".to_owned()),
            None,
            owner.clone(),
            1_000,
        );

        settlement.settle(StreamOutcome::Completed);

        assert_eq!(lock_bindings(&bindings).get("current"), Some(&vec![owner]));
    }
}
