//! Server-side shape policy — the declaration of what may ever be requested.
//!
//! ADR-009 requires the allowed practice, rows and columns to be derived **on the server**,
//! with client parameters unable to widen the approved shape. This module is that
//! declaration. It is deliberately data, not code: the set of clinical shapes is ASO's to
//! define (sequence step 1), so FRF loads it rather than hardcoding clinical schema.
//!
//! The widening defence is structural, in three parts:
//!
//! 1. **Columns are never client-supplied.** They come from the policy entry alone.
//! 2. **Parameters are allow-listed by name**, and an unknown key is an *error*, not an
//!    ignored field — a widening attempt fails loudly instead of silently succeeding with a
//!    broader result than the caller asked for.
//! 3. **The scope predicate is server-composed** and AND-ed with any narrowing, so a client
//!    filter can only ever intersect the authorized set.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::ShapeError;

/// One shape a client may ask for, as declared by server policy.
#[derive(Debug, Clone, Deserialize)]
pub struct ShapePolicy {
    /// Upstream table this shape reads.
    pub table: String,
    /// The exact columns a subject may see. Never widened by a request.
    pub columns: Vec<String>,
    /// Parameter names a client may narrow by. Any other key is rejected.
    #[serde(default)]
    pub allowed_params: Vec<String>,
    /// The Keto relation checked for this shape (e.g. `view`).
    pub relation: String,
    /// The Keto object namespace this shape's scope maps onto (e.g. `practice`).
    pub object_namespace: String,
}

/// The full set of declared shapes, keyed by shape id.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct ShapeCatalog {
    shapes: BTreeMap<String, ShapePolicy>,
}

impl ShapeCatalog {
    /// Build a catalog from declared entries.
    #[must_use]
    pub fn new(shapes: BTreeMap<String, ShapePolicy>) -> Self {
        Self { shapes }
    }

    /// Parse a catalog from JSON.
    ///
    /// # Errors
    ///
    /// [`ShapeError::Policy`] if the document is not a valid catalog.
    pub fn from_json(raw: &str) -> Result<Self, ShapeError> {
        serde_json::from_str(raw).map_err(|e| ShapeError::Policy(e.to_string()))
    }

    /// Look up a declared shape.
    ///
    /// # Errors
    ///
    /// [`ShapeError::UnknownShape`] if the id is not declared. An undeclared shape is refused
    /// outright rather than passed upstream — Electric never sees a request FRF did not
    /// sanction.
    pub fn get(&self, shape: &str) -> Result<&ShapePolicy, ShapeError> {
        self.shapes
            .get(shape)
            .ok_or_else(|| ShapeError::UnknownShape(shape.to_owned()))
    }

    /// Number of declared shapes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Whether the catalog declares no shapes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }
}

impl ShapePolicy {
    /// Compose the server-side row filter for `scope`, AND-ed with validated client
    /// narrowing.
    ///
    /// The scope predicate always leads, and every client term is AND-ed onto it, so the
    /// result can only ever be a subset of the authorized rows. A client cannot reach outside
    /// its scope by supplying an `OR`, because it never supplies an operator at all — only a
    /// value for an allow-listed key, which is quoted as a literal.
    ///
    /// # Errors
    ///
    /// [`ShapeError::ParamNotAllowed`] if a parameter key is not allow-listed.
    /// [`ShapeError::InvalidParam`] if a key or value is not a safe literal.
    pub fn compose_where(
        &self,
        scope_column: &str,
        scope_value: &str,
        params: &[(String, String)],
    ) -> Result<String, ShapeError> {
        validate_identifier(scope_column)?;
        let mut clause = format!("{scope_column} = {}", quote_literal(scope_value)?);

        for (key, value) in params {
            if !self.allowed_params.iter().any(|a| a == key) {
                // Loud failure, not a silent drop: a caller that believes it narrowed must
                // never receive a broader set than it asked for.
                return Err(ShapeError::ParamNotAllowed(key.clone()));
            }
            validate_identifier(key)?;
            clause.push_str(" AND ");
            clause.push_str(key);
            clause.push_str(" = ");
            clause.push_str(&quote_literal(value)?);
        }

        Ok(clause)
    }
}

/// Reject anything that is not a plain SQL identifier.
///
/// Identifiers here come from server policy and from allow-listed parameter names, never
/// from free-form client text — this is defence in depth so a mistaken policy edit cannot
/// become an injection vector.
fn validate_identifier(name: &str) -> Result<(), ShapeError> {
    let ok = !name.is_empty()
        && name.len() <= 63
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    if ok {
        Ok(())
    } else {
        Err(ShapeError::InvalidParam(format!(
            "not a valid identifier: {name}"
        )))
    }
}

/// Quote a value as a SQL string literal, doubling embedded quotes.
///
/// Control characters and NULs are refused rather than escaped: no legitimate scope or
/// narrowing value contains them, so their presence indicates a malformed or hostile input.
fn quote_literal(value: &str) -> Result<String, ShapeError> {
    if value.contains('\0') || value.chars().any(char::is_control) {
        return Err(ShapeError::InvalidParam(
            "value contains control characters".to_owned(),
        ));
    }
    if value.len() > 256 {
        return Err(ShapeError::InvalidParam("value too long".to_owned()));
    }
    Ok(format!("'{}'", value.replace('\'', "''")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ShapePolicy {
        ShapePolicy {
            table: "prior_auth_request".to_owned(),
            columns: vec!["id".to_owned(), "status".to_owned()],
            allowed_params: vec!["status".to_owned()],
            relation: "view".to_owned(),
            object_namespace: "practice".to_owned(),
        }
    }

    #[test]
    fn scope_predicate_leads_and_client_terms_only_narrow() {
        let clause = policy()
            .compose_where(
                "practice_id",
                "p-1",
                &[("status".to_owned(), "pending".to_owned())],
            )
            .expect("valid");
        assert_eq!(clause, "practice_id = 'p-1' AND status = 'pending'");
    }

    #[test]
    fn a_param_outside_the_allow_list_is_an_error_not_a_silent_drop() {
        // The widening case that matters: if this were ignored rather than refused, the
        // caller would receive every practice's rows while believing it had filtered.
        let err = policy()
            .compose_where(
                "practice_id",
                "p-1",
                &[("practice_id".to_owned(), "p-2".to_owned())],
            )
            .expect_err("must reject");
        assert!(matches!(err, ShapeError::ParamNotAllowed(k) if k == "practice_id"));
    }

    #[test]
    fn quote_injection_in_a_value_cannot_break_out_of_the_literal() {
        let clause = policy()
            .compose_where("practice_id", "p-1' OR '1'='1", &[])
            .expect("quoted, not rejected");
        // The payload stays one literal: doubled quotes, no free-standing OR.
        assert_eq!(clause, "practice_id = 'p-1'' OR ''1''=''1'");
        assert!(!clause.contains("OR '1'='1'"));
    }

    #[test]
    fn control_characters_in_a_value_are_refused() {
        let err = policy()
            .compose_where("practice_id", "p\u{0}1", &[])
            .expect_err("must reject");
        assert!(matches!(err, ShapeError::InvalidParam(_)));
    }

    #[test]
    fn a_non_identifier_scope_column_is_refused() {
        let err = policy()
            .compose_where("practice_id; DROP TABLE x", "p-1", &[])
            .expect_err("must reject");
        assert!(matches!(err, ShapeError::InvalidParam(_)));
    }

    #[test]
    fn an_undeclared_shape_is_refused_before_reaching_upstream() {
        let catalog = ShapeCatalog::default();
        let err = catalog.get("not-declared").expect_err("must reject");
        assert!(matches!(err, ShapeError::UnknownShape(s) if s == "not-declared"));
    }

    #[test]
    fn catalog_parses_from_json() {
        let catalog = ShapeCatalog::from_json(
            r#"{"prior_auth":{"table":"prior_auth_request","columns":["id"],
                "allowed_params":["status"],"relation":"view",
                "object_namespace":"practice"}}"#,
        )
        .expect("valid catalog");
        assert_eq!(catalog.len(), 1);
        assert_eq!(
            catalog.get("prior_auth").expect("present").table,
            "prior_auth_request"
        );
    }
}
