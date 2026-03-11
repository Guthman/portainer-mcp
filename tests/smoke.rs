//! Smoke test — run with PORTAINER_URL and PORTAINER_API_KEY set in the Run Configuration.

use portainer_mcp::client::PortainerClient;

#[tokio::test]
async fn list_endpoints_and_stacks() {
    let client = PortainerClient::new();

    let endpoints = client.list_endpoints(None, None, None, None).await.unwrap();
    println!("Endpoints ({}):", endpoints.len());
    for ep in &endpoints {
        println!(
            "  [{}] {} — {} (status {})",
            ep.id, ep.name, ep.url, ep.status
        );
    }
    assert!(!endpoints.is_empty(), "expected at least one endpoint");

    let stacks = client.list_stacks(None).await.unwrap();
    println!("\nStacks ({}):", stacks.len());
    for s in &stacks {
        println!(
            "  [{}] {} — type {} status {} endpoint {}",
            s.id, s.name, s.stack_type, s.status, s.endpoint_id
        );
    }
}
