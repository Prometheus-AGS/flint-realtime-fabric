use serde::{Deserialize, Serialize};

/// Body sent to `POST /relation-tuples/check` and `PUT /relation-tuples`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationTupleBody {
    pub namespace: String,
    pub object: String,
    pub relation: String,
    pub subject_id: String,
}

/// Response from `POST /relation-tuples/check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResponse {
    pub allowed: bool,
}

/// Body for `POST /relation-tuples/batch/check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCheckRequest {
    pub tuples: Vec<RelationTupleBody>,
}

/// Response from `POST /relation-tuples/batch/check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCheckResponse {
    pub results: Vec<CheckResponse>,
}
