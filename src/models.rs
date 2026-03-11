use std::collections::HashMap;

use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Deserializer, Serialize};

fn null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

// ── Response models ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Stack {
    #[serde(rename = "Id")]
    pub id: i64,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub stack_type: i64,
    #[serde(rename = "EndpointId")]
    pub endpoint_id: i64,
    #[serde(rename = "Status")]
    pub status: i64,
    #[serde(rename = "CreatedBy")]
    pub created_by: String,
    #[serde(rename = "CreationDate")]
    pub creation_date: i64,
    #[serde(rename = "UpdatedBy")]
    pub updated_by: String,
    #[serde(rename = "UpdateDate")]
    pub update_date: i64,
    #[serde(rename = "Env", deserialize_with = "null_as_empty_vec")]
    pub env: Vec<EnvVar>,
    #[serde(rename = "GitConfig")]
    pub git_config: Option<GitConfig>,
    #[serde(rename = "StackFileVersion")]
    pub stack_file_version: i64,
    #[serde(rename = "Webhook")]
    pub webhook: String,
    #[serde(rename = "EntryPoint")]
    pub entry_point: String,
    #[serde(rename = "ProjectPath")]
    pub project_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Endpoint {
    #[serde(rename = "Id")]
    pub id: i64,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub endpoint_type: i64,
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "Status")]
    pub status: i64,
    #[serde(rename = "GroupId")]
    pub group_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StackFileResponse {
    #[serde(rename = "StackFileContent")]
    pub stack_file_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnvVar {
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GitConfig {
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "ReferenceName")]
    pub reference_name: String,
    #[serde(rename = "ConfigFilePath")]
    pub config_file_path: String,
    #[serde(rename = "ConfigHash")]
    pub config_hash: String,
}

// ── Tool param structs ───────────────────────────────────────────────────────

/// Parameters for listing stacks.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListStacksParams {
    /// Optional JSON filter string, e.g. {"SwarmID":"abc","EndpointID":1}.
    pub filters: Option<String>,
}

/// Parameters for getting a single stack.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetStackParams {
    /// Stack identifier.
    pub id: i64,
}

/// Parameters for retrieving a stack's compose file content.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetStackFileParams {
    /// Stack identifier.
    pub id: i64,
    /// Stack file version maintained by Portainer.
    pub version: Option<i64>,
    /// Git repository commit hash. If provided alongside version, this takes precedence.
    pub commit_hash: Option<String>,
}

/// Parameters for creating a new standalone compose stack from a string.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateStackParams {
    /// Environment/endpoint identifier where the stack will be deployed.
    pub endpoint_id: i64,
    /// Name of the stack.
    pub name: String,
    /// Content of the docker-compose Stack file.
    pub stack_file_content: String,
    /// Optional environment variables as a list of {name, value} objects.
    pub env: Option<Vec<EnvVarInput>>,
    /// Optional webhook UUID for triggering redeployment.
    pub webhook: Option<String>,
}

/// Parameters for updating an existing stack.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateStackParams {
    /// Stack identifier.
    pub id: i64,
    /// Environment/endpoint identifier.
    pub endpoint_id: i64,
    /// New content of the Stack file.
    pub stack_file_content: Option<String>,
    /// Environment variables.
    pub env: Option<Vec<EnvVarInput>>,
    /// Prune services no longer referenced.
    pub prune: Option<bool>,
    /// Force repull images and redeploy.
    pub pull_image: Option<bool>,
    /// Stack file version to rollback to.
    pub rollback_to: Option<i64>,
}

/// Parameters for deleting a stack.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteStackParams {
    /// Stack identifier.
    pub id: i64,
    /// Environment/endpoint identifier.
    pub endpoint_id: i64,
    /// Whether to remove associated volumes.
    pub remove_volumes: Option<bool>,
}

/// Parameters for starting a stopped stack.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartStackParams {
    /// Stack identifier.
    pub id: i64,
    /// Environment/endpoint identifier.
    pub endpoint_id: i64,
}

/// Parameters for stopping a running stack.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StopStackParams {
    /// Stack identifier.
    pub id: i64,
    /// Environment/endpoint identifier.
    pub endpoint_id: i64,
}

/// Parameters for redeploying a git-based stack.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RedeployGitStackParams {
    /// Stack identifier.
    pub id: i64,
    /// Environment/endpoint identifier (required for legacy stacks).
    pub endpoint_id: Option<i64>,
    /// Environment variables.
    pub env: Option<Vec<EnvVarInput>>,
    /// Prune services no longer referenced.
    pub prune: Option<bool>,
    /// Git reference name (branch/tag) to deploy.
    pub repository_reference_name: Option<String>,
    /// Force repull images and redeploy.
    pub pull_image: Option<bool>,
}

/// Parameters for listing environments/endpoints.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListEndpointsParams {
    /// Start searching from this index.
    pub start: Option<i64>,
    /// Limit results to this number.
    pub limit: Option<i64>,
    /// Sort by field: Name, Group, Status.
    pub sort: Option<String>,
    /// Sort order: 1 for asc, 2 for desc.
    pub order: Option<i64>,
}

/// Parameters for making a generic Portainer API request.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenericRequestParams {
    /// HTTP method (GET, POST, PUT, DELETE, PATCH).
    pub method: String,
    /// API path after /api/, e.g. "status" or "endpoints/1/docker/containers/json".
    pub path: String,
    /// Optional JSON request body.
    pub body: Option<serde_json::Value>,
    /// Optional query string parameters.
    pub query_params: Option<HashMap<String, String>>,
}

/// Parameters for troubleshooting a stack.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TroubleshootStackParams {
    /// Stack identifier.
    pub stack_id: i64,
}

/// Parameters for the deploy guide prompt.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeployGuideParams {
    /// Environment/endpoint identifier to deploy to.
    pub endpoint_id: i64,
}

/// An environment variable with name and value.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnvVarInput {
    /// Variable name.
    pub name: String,
    /// Variable value.
    pub value: String,
}

// ── Request body structs ─────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CreateStackBody {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "StackFileContent")]
    pub stack_file_content: String,
    #[serde(rename = "Env", skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<EnvVarInput>>,
    #[serde(rename = "Webhook", skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateStackBody {
    #[serde(rename = "StackFileContent", skip_serializing_if = "Option::is_none")]
    pub stack_file_content: Option<String>,
    #[serde(rename = "Env", skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<EnvVarInput>>,
    #[serde(rename = "Prune", skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    #[serde(rename = "PullImage", skip_serializing_if = "Option::is_none")]
    pub pull_image: Option<bool>,
    #[serde(rename = "RollbackTo", skip_serializing_if = "Option::is_none")]
    pub rollback_to: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct RedeployGitStackBody {
    #[serde(rename = "Env", skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<EnvVarInput>>,
    #[serde(rename = "Prune", skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    #[serde(
        rename = "RepositoryReferenceName",
        skip_serializing_if = "Option::is_none"
    )]
    pub repository_reference_name: Option<String>,
    #[serde(rename = "PullImage", skip_serializing_if = "Option::is_none")]
    pub pull_image: Option<bool>,
}
