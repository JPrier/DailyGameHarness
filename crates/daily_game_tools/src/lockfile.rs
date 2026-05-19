use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
    pub schema_version: String,
    pub games: Vec<LockGame>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockGame {
    pub source_type: String,
    pub repo: Option<String>,
    pub requested_ref: Option<String>,
    pub resolved_sha: Option<String>,
    pub local_path: String,
}
