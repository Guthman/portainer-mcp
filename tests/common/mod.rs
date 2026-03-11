use std::time::Duration;

use portainer_mcp::client::PortainerClient;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor},
    runners::AsyncRunner,
};

pub struct PortainerTestInstance {
    pub client: PortainerClient,
    pub endpoint_id: i64,
    _container: ContainerAsync<GenericImage>,
}

impl PortainerTestInstance {
    pub async fn start() -> Self {
        let container = GenericImage::new("portainer/portainer-ce", "2.27.3")
            .with_exposed_port(9000.tcp())
            .with_wait_for(WaitFor::seconds(5))
            .with_mount(Mount::bind_mount(
                "/var/run/docker.sock",
                "/var/run/docker.sock",
            ))
            .start()
            .await
            .expect("failed to start Portainer container");

        let host = container.get_host().await.expect("failed to get host");
        let port = container
            .get_host_port_ipv4(9000)
            .await
            .expect("failed to get port");
        let base_url = format!("http://{}:{}", host, port);

        // Wait for Portainer to be ready
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("failed to build HTTP client");

        let status_url = format!("{}/api/system/status", base_url);
        for i in 0..60 {
            match http.get(&status_url).send().await {
                Ok(resp) if resp.status().is_success() => break,
                _ => {
                    if i == 59 {
                        panic!("Portainer did not become ready within 60 seconds");
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        // Create admin user
        let admin_body = serde_json::json!({
            "Username": "admin",
            "Password": "TestPass123!"
        });
        http.post(format!("{}/api/users/admin/init", base_url))
            .json(&admin_body)
            .send()
            .await
            .expect("failed to create admin user")
            .error_for_status()
            .expect("admin init returned error");

        // Authenticate to get JWT
        let auth_body = serde_json::json!({
            "Username": "admin",
            "Password": "TestPass123!"
        });
        let auth_resp: serde_json::Value = http
            .post(format!("{}/api/auth", base_url))
            .json(&auth_body)
            .send()
            .await
            .expect("failed to authenticate")
            .error_for_status()
            .expect("auth returned error")
            .json()
            .await
            .expect("failed to parse auth response");
        let jwt = auth_resp["jwt"].as_str().expect("no jwt in auth response");

        // Create API key
        let token_body = serde_json::json!({
            "description": "test-token",
            "password": "TestPass123!"
        });
        let token_resp: serde_json::Value = http
            .post(format!("{}/api/users/1/tokens", base_url))
            .bearer_auth(jwt)
            .json(&token_body)
            .send()
            .await
            .expect("failed to create API token")
            .error_for_status()
            .expect("token creation returned error")
            .json()
            .await
            .expect("failed to parse token response");
        let api_key = token_resp["rawAPIKey"]
            .as_str()
            .expect("no rawAPIKey in token response");

        // Create local Docker endpoint via form data
        let endpoint_resp: serde_json::Value = http
            .post(format!("{}/api/endpoints", base_url))
            .bearer_auth(jwt)
            .form(&[
                ("Name", "local"),
                ("EndpointCreationType", "1"),
                ("URL", "unix:///var/run/docker.sock"),
            ])
            .send()
            .await
            .expect("failed to create endpoint")
            .error_for_status()
            .expect("endpoint creation returned error")
            .json()
            .await
            .expect("failed to parse endpoint response");
        let endpoint_id = endpoint_resp["Id"]
            .as_i64()
            .expect("no Id in endpoint response");

        let client = PortainerClient::with_config(&base_url, api_key, false);

        Self {
            client,
            endpoint_id,
            _container: container,
        }
    }
}
