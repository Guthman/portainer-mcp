use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::client::PortainerClient;
use crate::models::*;

#[derive(Clone)]
pub struct PortainerServer {
    tool_router: ToolRouter<Self>,
    client: PortainerClient,
}

fn success_json<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("JSON serialization failed: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

fn err(msg: String) -> McpError {
    McpError::internal_error(msg, None)
}

impl Default for PortainerServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl PortainerServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            client: PortainerClient::new(),
        }
    }

    /// List all stacks in the Portainer instance. Returns stack id, name, type, status, endpoint, and git config.
    #[tool(
        description = "List all stacks. Optionally filter by SwarmID or EndpointID.\n\nArgs:\n  filters: Optional JSON filter string, e.g. {\"SwarmID\":\"abc\",\"EndpointID\":1}.\n\nReturns: Array of stack objects."
    )]
    async fn list_stacks(
        &self,
        Parameters(params): Parameters<ListStacksParams>,
    ) -> Result<CallToolResult, McpError> {
        let stacks = self
            .client
            .list_stacks(params.filters.as_deref())
            .await
            .map_err(err)?;
        success_json(&stacks)
    }

    /// Get details of a single stack by its identifier.
    #[tool(
        description = "Get a single stack by ID.\n\nArgs:\n  id: Stack identifier.\n\nReturns: Stack object with full details."
    )]
    async fn get_stack(
        &self,
        Parameters(params): Parameters<GetStackParams>,
    ) -> Result<CallToolResult, McpError> {
        let stack = self.client.get_stack(params.id).await.map_err(err)?;
        success_json(&stack)
    }

    /// Get the docker-compose file content of a stack.
    #[tool(
        description = "Get the compose file content of a stack.\n\nArgs:\n  id: Stack identifier.\n  version: Optional stack file version.\n  commit_hash: Optional git commit hash (takes precedence over version).\n\nReturns: The stack file content as a string."
    )]
    async fn get_stack_file(
        &self,
        Parameters(params): Parameters<GetStackFileParams>,
    ) -> Result<CallToolResult, McpError> {
        let file = self
            .client
            .get_stack_file(params.id, params.version, params.commit_hash.as_deref())
            .await
            .map_err(err)?;
        success_json(&file)
    }

    /// Create a new standalone docker-compose stack from a string.
    #[tool(
        description = "Create a new standalone compose stack from file content.\n\nArgs:\n  endpoint_id: Environment/endpoint ID for deployment.\n  name: Stack name.\n  stack_file_content: Docker-compose file content.\n  env: Optional environment variables [{name, value}, ...].\n  webhook: Optional webhook UUID.\n\nReturns: The created stack object."
    )]
    async fn create_stack(
        &self,
        Parameters(params): Parameters<CreateStackParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = CreateStackBody {
            name: params.name,
            stack_file_content: params.stack_file_content,
            env: params.env,
            webhook: params.webhook,
        };
        let stack = self
            .client
            .create_stack(params.endpoint_id, &body)
            .await
            .map_err(err)?;
        success_json(&stack)
    }

    /// Update an existing stack's compose file, environment, or settings.
    #[tool(
        description = "Update an existing stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n  stack_file_content: New compose file content.\n  env: Environment variables [{name, value}, ...].\n  prune: Prune services no longer referenced.\n  pull_image: Force repull images and redeploy.\n  rollback_to: Stack file version to rollback to.\n\nReturns: The updated stack object."
    )]
    async fn update_stack(
        &self,
        Parameters(params): Parameters<UpdateStackParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = UpdateStackBody {
            stack_file_content: params.stack_file_content,
            env: params.env,
            prune: params.prune,
            pull_image: params.pull_image,
            rollback_to: params.rollback_to,
        };
        let stack = self
            .client
            .update_stack(params.id, params.endpoint_id, &body)
            .await
            .map_err(err)?;
        success_json(&stack)
    }

    /// Delete a stack permanently.
    #[tool(
        description = "Delete a stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n  remove_volumes: Whether to remove associated volumes (default false).\n\nReturns: Confirmation message."
    )]
    async fn delete_stack(
        &self,
        Parameters(params): Parameters<DeleteStackParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_stack(params.id, params.endpoint_id, params.remove_volumes)
            .await
            .map_err(err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Stack {} deleted successfully.",
            params.id
        ))]))
    }

    /// Start a stopped stack.
    #[tool(
        description = "Start a stopped stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n\nReturns: The started stack object."
    )]
    async fn start_stack(
        &self,
        Parameters(params): Parameters<StartStackParams>,
    ) -> Result<CallToolResult, McpError> {
        let stack = self
            .client
            .start_stack(params.id, params.endpoint_id)
            .await
            .map_err(err)?;
        success_json(&stack)
    }

    /// Stop a running stack.
    #[tool(
        description = "Stop a running stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n\nReturns: The stopped stack object."
    )]
    async fn stop_stack(
        &self,
        Parameters(params): Parameters<StopStackParams>,
    ) -> Result<CallToolResult, McpError> {
        let stack = self
            .client
            .stop_stack(params.id, params.endpoint_id)
            .await
            .map_err(err)?;
        success_json(&stack)
    }

    /// Redeploy a git-based stack, optionally pulling latest changes.
    #[tool(
        description = "Redeploy a git-based stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Optional environment/endpoint ID (for legacy stacks).\n  env: Environment variables [{name, value}, ...].\n  prune: Prune services no longer referenced.\n  repository_reference_name: Git branch/tag to deploy.\n  pull_image: Force repull images and redeploy.\n\nReturns: The redeployed stack object."
    )]
    async fn redeploy_git_stack(
        &self,
        Parameters(params): Parameters<RedeployGitStackParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = RedeployGitStackBody {
            env: params.env,
            prune: params.prune,
            repository_reference_name: params.repository_reference_name,
            pull_image: params.pull_image,
        };
        let stack = self
            .client
            .redeploy_git_stack(params.id, params.endpoint_id, &body)
            .await
            .map_err(err)?;
        success_json(&stack)
    }

    /// List available environments/endpoints in Portainer.
    #[tool(
        description = "List environments/endpoints. Call this first to get endpoint IDs needed by other tools.\n\nArgs:\n  start: Start index.\n  limit: Max results.\n  sort: Sort field (Name, Group, Status).\n  order: Sort order (1=asc, 2=desc).\n\nReturns: Array of endpoint objects."
    )]
    async fn list_endpoints(
        &self,
        Parameters(params): Parameters<ListEndpointsParams>,
    ) -> Result<CallToolResult, McpError> {
        let endpoints = self
            .client
            .list_endpoints(
                params.start,
                params.limit,
                params.sort.as_deref(),
                params.order,
            )
            .await
            .map_err(err)?;
        success_json(&endpoints)
    }

    /// Make a generic Portainer API request for endpoints not covered by other tools.
    #[tool(
        description = "Make a generic Portainer API request. Use this for any endpoint not covered by the specific tools above.\n\nArgs:\n  method: HTTP method (GET, POST, PUT, DELETE, PATCH).\n  path: API path after /api/, e.g. \"status\" or \"endpoints/1/docker/containers/json\".\n  body: Optional JSON request body.\n  query_params: Optional query string parameters.\n\nReturns: JSON with status_code, headers, and body from the API response."
    )]
    async fn portainer_request(
        &self,
        Parameters(params): Parameters<GenericRequestParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .raw_request(
                &params.method,
                &params.path,
                params.body.as_ref(),
                params.query_params.as_ref(),
            )
            .await
            .map_err(err)?;
        let text = serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

#[tool_handler]
impl ServerHandler for PortainerServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Portainer stack management server. Manages Docker Compose stacks on a Portainer instance.\n\
                 \n\
                 Recommended workflow:\n\
                 1. Call list_endpoints first to get the endpoint_id for your environment.\n\
                 2. Call list_stacks to see available stacks.\n\
                 3. Use get_stack or get_stack_file to inspect a stack.\n\
                 4. Use create/update/delete/start/stop/redeploy tools to manage stacks.\n\
                 5. Use portainer_request for any Portainer API endpoint not covered above."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
