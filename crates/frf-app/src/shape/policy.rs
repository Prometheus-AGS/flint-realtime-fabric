//! Server-owned shape declarations and narrowing rules.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::ShapeUseCaseError;

const APPROVED_REFERENCE_PROJECTIONS: [(&str, &str, [&str; 3]); 1] = [(
    "evidence_states",
    "aso.evidence_states",
    ["key", "label", "meaning"],
)];

/// One shape a client may request, as declared by server policy.
#[derive(Debug, Clone, Deserialize)]
pub struct ShapePolicy {
    /// Upstream base table.
    pub table: String,
    /// Exact server-approved columns.
    pub columns: Vec<String>,
    /// Optional client keys that may narrow the server-owned predicate.
    #[serde(default)]
    pub allowed_params: Vec<String>,
    /// Authorization relation evaluated for every request.
    pub relation: String,
    /// Authorization object namespace for the verified practice.
    pub object_namespace: String,
    /// Whether the projection is a shared server-owned reference table.
    #[serde(default)]
    pub reference: bool,
    /// Row column carrying the verified practice. It may be omitted only when
    /// `reference` is explicitly true.
    #[serde(default)]
    pub scope_column: Option<String>,
}

/// Server-owned catalog keyed by public shape identifier.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct ShapeCatalog {
    shapes: BTreeMap<String, ShapePolicy>,
}

impl ShapeCatalog {
    /// Parse a catalog from deployment JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeUseCaseError::InvalidRequest`] for invalid policy JSON.
    pub fn from_json(raw: &str) -> Result<Self, ShapeUseCaseError> {
        let catalog: Self = serde_json::from_str(raw)
            .map_err(|error| ShapeUseCaseError::InvalidRequest(error.to_string()))?;
        for (shape, policy) in &catalog.shapes {
            match (policy.reference, policy.scope_column.as_deref()) {
                (false, None) => {
                    return Err(ShapeUseCaseError::InvalidRequest(format!(
                        "shape {shape} must declare a scope column or be an explicit reference"
                    )));
                }
                (true, Some(_)) => {
                    return Err(ShapeUseCaseError::InvalidRequest(format!(
                        "reference shape {shape} cannot declare a scope column"
                    )));
                }
                (true, None) if !approved_reference(shape, policy) => {
                    return Err(ShapeUseCaseError::InvalidRequest(format!(
                        "shape {shape} is not an approved reference projection"
                    )));
                }
                _ => {}
            }
        }
        Ok(catalog)
    }

    /// Look up one declared shape.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeUseCaseError::UnknownShape`] when policy does not declare
    /// the public identifier.
    pub fn get(&self, shape: &str) -> Result<&ShapePolicy, ShapeUseCaseError> {
        self.shapes
            .get(shape)
            .ok_or(ShapeUseCaseError::UnknownShape)
    }

    /// Number of declared shapes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Whether no shape has been declared.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }
}

fn approved_reference(shape: &str, policy: &ShapePolicy) -> bool {
    APPROVED_REFERENCE_PROJECTIONS.iter().any(
        |(approved_shape, approved_table, approved_columns)| {
            shape == *approved_shape
                && policy.table == *approved_table
                && policy.columns.len() == approved_columns.len()
                && policy
                    .columns
                    .iter()
                    .zip(approved_columns)
                    .all(|(actual, approved)| actual == approved)
                && policy.allowed_params.is_empty()
                && policy.relation == "view"
                && policy.object_namespace == "practice"
        },
    )
}

impl ShapePolicy {
    pub(super) fn compose_where(
        &self,
        scope_value: &str,
        params: &[(String, String)],
    ) -> Result<Option<String>, ShapeUseCaseError> {
        let mut predicates = Vec::new();
        if let Some(scope_column) = &self.scope_column {
            validate_identifier(scope_column)?;
            predicates.push(format!("{scope_column} = {}", quote_literal(scope_value)?));
        }
        for (key, value) in params {
            if !self.allowed_params.iter().any(|allowed| allowed == key) {
                return Err(ShapeUseCaseError::InvalidRequest(format!(
                    "parameter not allowed: {key}"
                )));
            }
            validate_identifier(key)?;
            predicates.push(format!("{key} = {}", quote_literal(value)?));
        }
        Ok((!predicates.is_empty()).then(|| predicates.join(" AND ")))
    }
}

fn validate_identifier(name: &str) -> Result<(), ShapeUseCaseError> {
    let valid = !name.is_empty()
        && name.len() <= 63
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '.'))
        && name
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic());
    if valid {
        Ok(())
    } else {
        Err(ShapeUseCaseError::InvalidRequest(
            "invalid server policy identifier".to_owned(),
        ))
    }
}

fn quote_literal(value: &str) -> Result<String, ShapeUseCaseError> {
    if value.contains('\0') || value.chars().any(char::is_control) || value.len() > 256 {
        return Err(ShapeUseCaseError::InvalidRequest(
            "invalid narrowing value".to_owned(),
        ));
    }
    Ok(format!("'{}'", value.replace('\'', "''")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ShapePolicy {
        ShapePolicy {
            table: "aso.cases".to_owned(),
            columns: vec!["id".to_owned()],
            allowed_params: vec!["status".to_owned()],
            relation: "view".to_owned(),
            object_namespace: "practice".to_owned(),
            reference: false,
            scope_column: Some("practice_id".to_owned()),
        }
    }

    #[test]
    fn scope_leads_and_allowed_values_can_only_narrow() {
        let clause = policy()
            .compose_where("practice-1", &[("status".to_owned(), "pending".to_owned())])
            .expect("valid policy");
        assert_eq!(
            clause.as_deref(),
            Some("practice_id = 'practice-1' AND status = 'pending'")
        );
    }

    #[test]
    fn unknown_parameter_is_rejected_instead_of_ignored() {
        let error = policy()
            .compose_where(
                "practice-1",
                &[("practice_id".to_owned(), "practice-2".to_owned())],
            )
            .expect_err("scope cannot be replaced");
        assert!(matches!(error, ShapeUseCaseError::InvalidRequest(_)));
    }

    #[test]
    fn literal_quoting_keeps_an_or_payload_inside_one_value() {
        let clause = policy()
            .compose_where("p-1' OR '1'='1", &[])
            .expect("quoted literal");
        assert_eq!(
            clause.as_deref(),
            Some("practice_id = 'p-1'' OR ''1''=''1'")
        );
    }

    #[test]
    fn approved_reference_without_narrowing_omits_where_clause() {
        let reference = ShapePolicy {
            table: "aso.evidence_states".to_owned(),
            columns: vec!["key".to_owned(), "label".to_owned(), "meaning".to_owned()],
            allowed_params: Vec::new(),
            relation: "view".to_owned(),
            object_namespace: "practice".to_owned(),
            reference: true,
            scope_column: None,
        };
        assert_eq!(
            reference.compose_where("practice-1", &[]).expect("policy"),
            None
        );
    }

    #[test]
    fn missing_scope_is_rejected_without_an_explicit_reference_declaration() {
        let error = ShapeCatalog::from_json(
            r#"{"cases":{"table":"aso.cases","columns":["id"],"allowed_params":[],"relation":"view","object_namespace":"practice"}}"#,
        )
        .expect_err("practice data must never become an implicit reference projection");
        assert!(
            matches!(error, ShapeUseCaseError::InvalidRequest(message) if message.contains("must declare a scope column"))
        );
    }

    #[test]
    fn explicit_reference_without_scope_is_accepted() {
        let catalog = ShapeCatalog::from_json(
            r#"{"evidence_states":{"table":"aso.evidence_states","columns":["key","label","meaning"],"allowed_params":[],"relation":"view","object_namespace":"practice","reference":true}}"#,
        )
        .expect("explicit reference projection");
        assert_eq!(catalog.len(), 1);
    }

    #[test]
    fn practice_table_cannot_self_declare_as_a_reference_projection() {
        let error = ShapeCatalog::from_json(
            r#"{"cases":{"table":"aso.cases","columns":["id","practice_id","status"],"allowed_params":[],"relation":"view","object_namespace":"practice","reference":true}}"#,
        )
        .expect_err("practice-owned table must retain its scope predicate");
        assert!(
            matches!(error, ShapeUseCaseError::InvalidRequest(message) if message.contains("not an approved reference projection"))
        );
    }
}
