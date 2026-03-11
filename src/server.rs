use std::sync::{Arc, RwLock};

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::{
        AnnotateAble, CallToolResult, Content, GetPromptRequestParam, GetPromptResult,
        ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult, LoggingLevel,
        LoggingMessageNotificationParam, PaginatedRequestParam, PromptMessage, PromptMessageRole,
        RawResource, RawResourceTemplate, ReadResourceRequestParam, ReadResourceResult,
        ResourceContents, ServerCapabilities, ServerInfo, SetLevelRequestParam,
    },
    prompt, prompt_handler, prompt_router,
    service::{Peer, RequestContext, RoleServer},
    tool, tool_handler, tool_router,
};

use crate::client::PortainerClient;
use crate::models::*;

#[derive(Clone)]
pub struct PortainerServer {
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
    client: PortainerClient,
    log_level: Arc<RwLock<Option<LoggingLevel>>>,
}

fn success_json<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("JSON serialization failed: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

fn err(msg: String) -> McpError {
    McpError::internal_error(msg, None)
}

fn log_level_ordinal(level: LoggingLevel) -> u8 {
    match level {
        LoggingLevel::Debug => 0,
        LoggingLevel::Info => 1,
        LoggingLevel::Notice => 2,
        LoggingLevel::Warning => 3,
        LoggingLevel::Error => 4,
        LoggingLevel::Critical => 5,
        LoggingLevel::Alert => 6,
        LoggingLevel::Emergency => 7,
    }
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
            prompt_router: Self::prompt_router(),
            client: PortainerClient::new(),
            log_level: Arc::new(RwLock::new(None)),
        }
    }

    /// List all stacks in the Portainer instance. Returns stack id, name, type, status, endpoint, and git config.
    #[tool(
        description = "List all stacks. Optionally filter by SwarmID or EndpointID.\n\nArgs:\n  filters: Optional JSON filter string, e.g. {\"SwarmID\":\"abc\",\"EndpointID\":1}.\n\nReturns: Array of stack objects.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Get a single stack by ID.\n\nArgs:\n  id: Stack identifier.\n\nReturns: Stack object with full details.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Get the compose file content of a stack.\n\nArgs:\n  id: Stack identifier.\n  version: Optional stack file version.\n  commit_hash: Optional git commit hash (takes precedence over version).\n\nReturns: The stack file content as a string.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Create a new standalone compose stack from file content.\n\nArgs:\n  endpoint_id: Environment/endpoint ID for deployment.\n  name: Stack name.\n  stack_file_content: Docker-compose file content.\n  env: Optional environment variables [{name, value}, ...].\n  webhook: Optional webhook UUID.\n\nReturns: The created stack object.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
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
        description = "Update an existing stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n  stack_file_content: New compose file content.\n  env: Environment variables [{name, value}, ...].\n  prune: Prune services no longer referenced.\n  pull_image: Force repull images and redeploy.\n  rollback_to: Stack file version to rollback to.\n\nReturns: The updated stack object.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Delete a stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n  remove_volumes: Whether to remove associated volumes (default false).\n\nReturns: Confirmation message.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
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
        description = "Start a stopped stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n\nReturns: The started stack object.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Stop a running stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Environment/endpoint ID.\n\nReturns: The stopped stack object.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Redeploy a git-based stack.\n\nArgs:\n  id: Stack identifier.\n  endpoint_id: Optional environment/endpoint ID (for legacy stacks).\n  env: Environment variables [{name, value}, ...].\n  prune: Prune services no longer referenced.\n  repository_reference_name: Git branch/tag to deploy.\n  pull_image: Force repull images and redeploy.\n\nReturns: The redeployed stack object.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
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
        description = "List environments/endpoints. Call this first to get endpoint IDs needed by other tools.\n\nArgs:\n  start: Start index.\n  limit: Max results.\n  sort: Sort field (Name, Group, Status).\n  order: Sort order (1=asc, 2=desc).\n\nReturns: Array of endpoint objects.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Make a generic Portainer API request. Use this for any endpoint not covered by the specific tools above.\n\nArgs:\n  method: HTTP method (GET, POST, PUT, DELETE, PATCH).\n  path: API path after /api/, e.g. \"status\" or \"endpoints/1/docker/containers/json\".\n  body: Optional JSON request body.\n  query_params: Optional query string parameters.\n\nReturns: JSON with status_code, headers, and body from the API response.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
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

// ── Prompts ──────────────────────────────────────────────────────────────────

#[prompt_router]
impl PortainerServer {
    #[prompt(
        name = "troubleshoot-stack",
        description = "Fetch stack details and compose file, then guide troubleshooting."
    )]
    async fn troubleshoot_stack(
        &self,
        Parameters(params): Parameters<TroubleshootStackParams>,
    ) -> Result<GetPromptResult, McpError> {
        let stack_info = match self.client.get_stack(params.stack_id).await {
            Ok(s) => {
                let status_label = match s.status {
                    1 => "active",
                    2 => "inactive",
                    _ => "unknown",
                };
                let git_info = s.git_config.as_ref().map_or_else(
                    || "Not a git-based stack".to_string(),
                    |g| {
                        format!(
                            "Git: {} ref={} path={}",
                            g.url, g.reference_name, g.config_file_path
                        )
                    },
                );
                format!(
                    "Name: {}\nID: {}\nType: {}\nStatus: {} ({})\nEndpoint ID: {}\n\
                     Created by: {}\nEntry point: {}\n{}",
                    s.name,
                    s.id,
                    s.stack_type,
                    s.status,
                    status_label,
                    s.endpoint_id,
                    s.created_by,
                    s.entry_point,
                    git_info
                )
            }
            Err(e) => format!("Error fetching stack {}: {}", params.stack_id, e),
        };

        let compose_info = match self
            .client
            .get_stack_file(params.stack_id, None, None)
            .await
        {
            Ok(f) => f.stack_file_content,
            Err(e) => format!("Error fetching compose file: {}", e),
        };

        let message = format!(
            "Please help troubleshoot the following Portainer stack.\n\n\
             ## Stack Details\n{stack_info}\n\n\
             ## Compose File\n```yaml\n{compose_info}\n```\n\n\
             ## Diagnostic Checklist\n\
             Please analyze the stack for:\n\
             1. Configuration issues (invalid YAML, missing required fields)\n\
             2. Networking problems (port conflicts, missing networks)\n\
             3. Resource constraints (memory/CPU limits, volume mounts)\n\
             4. Environment variables (missing or misconfigured)\n\
             5. Git configuration issues (if git-based stack)\n\
             6. Best practices (image tags, restart policies, health checks)"
        );

        Ok(GetPromptResult {
            description: Some("Troubleshoot a Portainer stack".to_string()),
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, message)],
        })
    }

    #[prompt(
        name = "deploy-guide",
        description = "Guide through deploying a new stack on a specific endpoint."
    )]
    async fn deploy_guide(
        &self,
        Parameters(params): Parameters<DeployGuideParams>,
    ) -> Result<GetPromptResult, McpError> {
        let endpoint_info = match self.client.list_endpoints(None, None, None, None).await {
            Ok(endpoints) => {
                let target = endpoints.iter().find(|e| e.id == params.endpoint_id);
                match target {
                    Some(ep) => format!(
                        "Target endpoint: {} (ID: {}, URL: {}, Status: {})",
                        ep.name, ep.id, ep.url, ep.status
                    ),
                    None => {
                        let available: Vec<String> = endpoints
                            .iter()
                            .map(|e| format!("  - {} (ID: {})", e.name, e.id))
                            .collect();
                        format!(
                            "Endpoint {} not found. Available endpoints:\n{}",
                            params.endpoint_id,
                            available.join("\n")
                        )
                    }
                }
            }
            Err(e) => format!("Error fetching endpoints: {}", e),
        };

        let message = format!(
            "Please guide me through deploying a new Docker Compose stack on Portainer.\n\n\
             ## Environment\n{endpoint_info}\n\n\
             ## Deployment Steps\n\
             To create a new stack, use the `create_stack` tool with these parameters:\n\
             - `endpoint_id`: {}\n\
             - `name`: A unique name for your stack\n\
             - `stack_file_content`: Your Docker Compose YAML content\n\
             - `env` (optional): Environment variables as [{{\"name\": \"KEY\", \"value\": \"VAL\"}}]\n\
             - `webhook` (optional): UUID for webhook-triggered redeployment\n\n\
             Please provide your Docker Compose file content and I'll help you deploy it.",
            params.endpoint_id
        );

        Ok(GetPromptResult {
            description: Some("Guide for deploying a new stack".to_string()),
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, message)],
        })
    }

    #[prompt(
        name = "stack-overview",
        description = "Fetch all endpoints and stacks for a full infrastructure overview."
    )]
    async fn stack_overview(&self) -> Result<GetPromptResult, McpError> {
        let endpoints_info = match self.client.list_endpoints(None, None, None, None).await {
            Ok(endpoints) => {
                if endpoints.is_empty() {
                    "No endpoints configured.".to_string()
                } else {
                    endpoints
                        .iter()
                        .map(|e| {
                            format!(
                                "- {} (ID: {}, URL: {}, Status: {})",
                                e.name, e.id, e.url, e.status
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Err(e) => format!("Error fetching endpoints: {}", e),
        };

        let stacks_info = match self.client.list_stacks(None).await {
            Ok(stacks) => {
                if stacks.is_empty() {
                    "No stacks deployed.".to_string()
                } else {
                    stacks
                        .iter()
                        .map(|s| {
                            let status = match s.status {
                                1 => "active",
                                2 => "inactive",
                                _ => "unknown",
                            };
                            format!(
                                "- {} (ID: {}, Status: {}, Endpoint: {})",
                                s.name, s.id, status, s.endpoint_id
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Err(e) => format!("Error fetching stacks: {}", e),
        };

        let message = format!(
            "Please provide a comprehensive overview of my Portainer infrastructure.\n\n\
             ## Endpoints\n{endpoints_info}\n\n\
             ## Stacks\n{stacks_info}\n\n\
             ## Requested Analysis\n\
             1. Infrastructure summary and health assessment\n\
             2. Stack status review (any stopped or problematic stacks)\n\
             3. Recommendations for improvements or issues to address"
        );

        Ok(GetPromptResult {
            description: Some("Full infrastructure overview".to_string()),
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, message)],
        })
    }
}

// ── Logging helper ───────────────────────────────────────────────────────────

impl PortainerServer {
    async fn send_log(
        &self,
        peer: &Peer<RoleServer>,
        level: LoggingLevel,
        logger: &str,
        data: serde_json::Value,
    ) {
        let min_level = {
            let guard = self.log_level.read().expect("log_level lock poisoned");
            match *guard {
                Some(l) => l,
                None => return,
            }
        };
        if log_level_ordinal(level) >= log_level_ordinal(min_level) {
            let _ = peer
                .notify_logging_message(LoggingMessageNotificationParam {
                    level,
                    logger: Some(logger.to_string()),
                    data,
                })
                .await;
        }
    }
}

// ── ServerHandler ────────────────────────────────────────────────────────────

#[tool_handler]
#[prompt_handler]
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
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .enable_logging()
                .build(),
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        self.send_log(
            &context.peer,
            LoggingLevel::Debug,
            "resources",
            serde_json::json!("Listing resources"),
        )
        .await;

        let endpoints = RawResource {
            uri: "portainer://endpoints".to_string(),
            name: "endpoints".to_string(),
            title: None,
            description: Some("All environments/endpoints".to_string()),
            mime_type: Some("application/json".to_string()),
            size: None,
            icons: None,
        }
        .no_annotation();

        let stacks = RawResource {
            uri: "portainer://stacks".to_string(),
            name: "stacks".to_string(),
            title: None,
            description: Some("All stacks".to_string()),
            mime_type: Some("application/json".to_string()),
            size: None,
            icons: None,
        }
        .no_annotation();

        Ok(ListResourcesResult {
            next_cursor: None,
            resources: vec![endpoints, stacks],
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        self.send_log(
            &context.peer,
            LoggingLevel::Debug,
            "resources",
            serde_json::json!("Listing resource templates"),
        )
        .await;

        let stack = RawResourceTemplate {
            uri_template: "portainer://stacks/{stack_id}".to_string(),
            name: "stack".to_string(),
            title: None,
            description: Some("Single stack details".to_string()),
            mime_type: Some("application/json".to_string()),
        }
        .no_annotation();

        let compose = RawResourceTemplate {
            uri_template: "portainer://stacks/{stack_id}/compose".to_string(),
            name: "stack-compose".to_string(),
            title: None,
            description: Some("Stack compose file".to_string()),
            mime_type: Some("application/x-yaml".to_string()),
        }
        .no_annotation();

        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: vec![stack, compose],
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;

        self.send_log(
            &context.peer,
            LoggingLevel::Info,
            "resources",
            serde_json::json!(format!("Reading resource: {uri}")),
        )
        .await;

        if uri == "portainer://endpoints" {
            let endpoints = self
                .client
                .list_endpoints(None, None, None, None)
                .await
                .map_err(err)?;
            let text = serde_json::to_string_pretty(&endpoints)
                .map_err(|e| err(format!("JSON serialization failed: {e}")))?;
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::TextResourceContents {
                    uri: uri.clone(),
                    mime_type: Some("application/json".to_string()),
                    text,
                    meta: None,
                }],
            })
        } else if uri == "portainer://stacks" {
            let stacks = self.client.list_stacks(None).await.map_err(err)?;
            let text = serde_json::to_string_pretty(&stacks)
                .map_err(|e| err(format!("JSON serialization failed: {e}")))?;
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::TextResourceContents {
                    uri: uri.clone(),
                    mime_type: Some("application/json".to_string()),
                    text,
                    meta: None,
                }],
            })
        } else if let Some(rest) = uri.strip_prefix("portainer://stacks/") {
            if let Some(id_str) = rest.strip_suffix("/compose") {
                let id: i64 = id_str.parse().map_err(|_| {
                    McpError::invalid_params(format!("Invalid stack ID: {id_str}"), None)
                })?;
                let file = self
                    .client
                    .get_stack_file(id, None, None)
                    .await
                    .map_err(err)?;
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::TextResourceContents {
                        uri: uri.clone(),
                        mime_type: Some("application/x-yaml".to_string()),
                        text: file.stack_file_content,
                        meta: None,
                    }],
                })
            } else {
                let id: i64 = rest.parse().map_err(|_| {
                    McpError::invalid_params(format!("Invalid stack ID: {rest}"), None)
                })?;
                let stack = self.client.get_stack(id).await.map_err(err)?;
                let text = serde_json::to_string_pretty(&stack)
                    .map_err(|e| err(format!("JSON serialization failed: {e}")))?;
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::TextResourceContents {
                        uri: uri.clone(),
                        mime_type: Some("application/json".to_string()),
                        text,
                        meta: None,
                    }],
                })
            }
        } else {
            Err(McpError::invalid_params(
                format!("Resource not found: {uri}"),
                None,
            ))
        }
    }

    async fn set_level(
        &self,
        request: SetLevelRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        let mut guard = self.log_level.write().expect("log_level lock poisoned");
        *guard = Some(request.level);
        Ok(())
    }
}
