use std::collections::HashMap;

use reqwest::{Client, Method, RequestBuilder};

use crate::models::*;

#[derive(Clone)]
pub struct PortainerClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl Default for PortainerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl PortainerClient {
    pub fn new() -> Self {
        let api_key = std::env::var("PORTAINER_API_KEY").expect("PORTAINER_API_KEY must be set");
        let base_url = std::env::var("PORTAINER_URL")
            .unwrap_or_else(|_| "http://localhost:9000".into())
            .trim_end_matches('/')
            .to_string();

        let insecure = std::env::var("PORTAINER_INSECURE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .danger_accept_invalid_certs(insecure)
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            base_url,
            api_key,
        }
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}/api/{}", self.base_url, path.trim_start_matches('/'));
        self.client
            .request(method, &url)
            .header("X-API-KEY", &self.api_key)
    }

    async fn check_response(response: reqwest::Response) -> Result<String, String> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("failed to read response body: {e}"))?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(format!("Portainer API error (HTTP {status}): {body}"))
        }
    }

    // ── Typed methods ────────────────────────────────────────────────────────

    pub async fn list_stacks(&self, filters: Option<&str>) -> Result<Vec<Stack>, String> {
        let mut req = self.request(Method::GET, "stacks");
        if let Some(f) = filters {
            req = req.query(&[("filters", f)]);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let body = Self::check_response(resp).await?;
        serde_json::from_str(&body).map_err(|e| format!("failed to parse stacks: {e}"))
    }

    pub async fn get_stack(&self, id: i64) -> Result<Stack, String> {
        let resp = self
            .request(Method::GET, &format!("stacks/{id}"))
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let body = Self::check_response(resp).await?;
        serde_json::from_str(&body).map_err(|e| format!("failed to parse stack: {e}"))
    }

    pub async fn get_stack_file(
        &self,
        id: i64,
        version: Option<i64>,
        commit_hash: Option<&str>,
    ) -> Result<StackFileResponse, String> {
        let mut req = self.request(Method::GET, &format!("stacks/{id}/file"));
        if let Some(v) = version {
            req = req.query(&[("version", v.to_string())]);
        }
        if let Some(h) = commit_hash {
            req = req.query(&[("commitHash", h)]);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let body = Self::check_response(resp).await?;
        serde_json::from_str(&body).map_err(|e| format!("failed to parse stack file: {e}"))
    }

    pub async fn create_stack(
        &self,
        endpoint_id: i64,
        body: &CreateStackBody,
    ) -> Result<Stack, String> {
        let resp = self
            .request(Method::POST, "stacks/create/standalone/string")
            .query(&[("endpointId", endpoint_id)])
            .json(body)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let resp_body = Self::check_response(resp).await?;
        serde_json::from_str(&resp_body).map_err(|e| format!("failed to parse created stack: {e}"))
    }

    pub async fn update_stack(
        &self,
        id: i64,
        endpoint_id: i64,
        body: &UpdateStackBody,
    ) -> Result<Stack, String> {
        let resp = self
            .request(Method::PUT, &format!("stacks/{id}"))
            .query(&[("endpointId", endpoint_id)])
            .json(body)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let resp_body = Self::check_response(resp).await?;
        serde_json::from_str(&resp_body).map_err(|e| format!("failed to parse updated stack: {e}"))
    }

    pub async fn delete_stack(
        &self,
        id: i64,
        endpoint_id: i64,
        remove_volumes: Option<bool>,
    ) -> Result<(), String> {
        let mut req = self
            .request(Method::DELETE, &format!("stacks/{id}"))
            .query(&[("endpointId", endpoint_id)]);
        if let Some(rv) = remove_volumes {
            req = req.query(&[("removeVolumes", rv)]);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        Self::check_response(resp).await?;
        Ok(())
    }

    pub async fn start_stack(&self, id: i64, endpoint_id: i64) -> Result<Stack, String> {
        let resp = self
            .request(Method::POST, &format!("stacks/{id}/start"))
            .query(&[("endpointId", endpoint_id)])
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let resp_body = Self::check_response(resp).await?;
        serde_json::from_str(&resp_body).map_err(|e| format!("failed to parse stack: {e}"))
    }

    pub async fn stop_stack(&self, id: i64, endpoint_id: i64) -> Result<Stack, String> {
        let resp = self
            .request(Method::POST, &format!("stacks/{id}/stop"))
            .query(&[("endpointId", endpoint_id)])
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let resp_body = Self::check_response(resp).await?;
        serde_json::from_str(&resp_body).map_err(|e| format!("failed to parse stack: {e}"))
    }

    pub async fn redeploy_git_stack(
        &self,
        id: i64,
        endpoint_id: Option<i64>,
        body: &RedeployGitStackBody,
    ) -> Result<Stack, String> {
        let mut req = self
            .request(Method::PUT, &format!("stacks/{id}/git/redeploy"))
            .json(body);
        if let Some(eid) = endpoint_id {
            req = req.query(&[("endpointId", eid)]);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let resp_body = Self::check_response(resp).await?;
        serde_json::from_str(&resp_body)
            .map_err(|e| format!("failed to parse redeployed stack: {e}"))
    }

    pub async fn list_endpoints(
        &self,
        start: Option<i64>,
        limit: Option<i64>,
        sort: Option<&str>,
        order: Option<i64>,
    ) -> Result<Vec<Endpoint>, String> {
        let mut req = self.request(Method::GET, "endpoints");
        if let Some(s) = start {
            req = req.query(&[("start", s.to_string())]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }
        if let Some(s) = sort {
            req = req.query(&[("sort", s)]);
        }
        if let Some(o) = order {
            req = req.query(&[("order", o.to_string())]);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let body = Self::check_response(resp).await?;
        serde_json::from_str(&body).map_err(|e| format!("failed to parse endpoints: {e}"))
    }

    pub async fn raw_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<serde_json::Value, String> {
        let method: Method = method
            .to_uppercase()
            .parse()
            .map_err(|e| format!("invalid HTTP method: {e}"))?;

        let mut req = self.request(method, path);
        if let Some(b) = body {
            // Handle LLM passing body as a JSON-encoded string
            let b = if let serde_json::Value::String(s) = b {
                serde_json::from_str(s).unwrap_or_else(|_| b.clone())
            } else {
                b.clone()
            };
            req = req.json(&b);
        }
        if let Some(qp) = query_params {
            req = req.query(qp);
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let status_code = response.status().as_u16();
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<binary>").to_string()))
            .collect();

        let body_text = response
            .text()
            .await
            .map_err(|e| format!("failed to read response body: {e}"))?;

        let body_value: serde_json::Value =
            serde_json::from_str(&body_text).unwrap_or(serde_json::Value::String(body_text));

        Ok(serde_json::json!({
            "status_code": status_code,
            "headers": response_headers,
            "body": body_value,
        }))
    }
}
