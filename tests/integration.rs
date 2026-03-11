mod common;

use portainer_mcp::models::{CreateStackBody, UpdateStackBody};

const COMPOSE_FILE: &str = r#"version: "3"
services:
  web:
    image: nginx:alpine
    ports:
      - "80:80"
"#;

#[tokio::test]
async fn test_list_endpoints() {
    let instance = common::PortainerTestInstance::start().await;

    let endpoints = instance
        .client
        .list_endpoints(None, None, None, None)
        .await
        .expect("list_endpoints failed");

    assert!(!endpoints.is_empty(), "expected at least one endpoint");
    assert_eq!(endpoints[0].name, "local");
    assert_eq!(endpoints[0].id, 1);
}

#[tokio::test]
async fn test_list_stacks_empty() {
    let instance = common::PortainerTestInstance::start().await;

    let stacks = instance
        .client
        .list_stacks(None)
        .await
        .expect("list_stacks failed");

    assert!(stacks.is_empty(), "expected no stacks on fresh instance");
}

#[tokio::test]
async fn test_generic_request_system_status() {
    let instance = common::PortainerTestInstance::start().await;

    let result = instance
        .client
        .raw_request("GET", "system/status", None, None)
        .await
        .expect("raw_request failed");

    assert_eq!(result["status_code"], 200);
    assert!(
        result["body"]["Version"].is_string(),
        "expected Version field in response"
    );
}

#[tokio::test]
async fn test_stack_lifecycle() {
    let instance = common::PortainerTestInstance::start().await;
    let eid = instance.endpoint_id;

    // 1. Create stack
    let create_body = CreateStackBody {
        name: "test-nginx".to_string(),
        stack_file_content: COMPOSE_FILE.to_string(),
        env: None,
        webhook: None,
    };
    let created = instance
        .client
        .create_stack(eid, &create_body)
        .await
        .expect("create_stack failed");
    assert_eq!(created.name, "test-nginx");
    assert_eq!(created.endpoint_id, eid);
    let stack_id = created.id;

    // 2. List stacks — should have 1
    let stacks = instance
        .client
        .list_stacks(None)
        .await
        .expect("list_stacks failed");
    assert_eq!(stacks.len(), 1);

    // 3. Get stack
    let stack = instance
        .client
        .get_stack(stack_id)
        .await
        .expect("get_stack failed");
    assert_eq!(stack.id, stack_id);
    assert_eq!(stack.name, "test-nginx");

    // 4. Get stack file
    let file = instance
        .client
        .get_stack_file(stack_id, None, None)
        .await
        .expect("get_stack_file failed");
    assert!(
        file.stack_file_content.contains("nginx:alpine"),
        "expected nginx:alpine in stack file"
    );

    // 5. Update stack — change port mapping
    let updated_compose = "version: \"3\"\nservices:\n  web:\n    image: nginx:alpine\n    ports:\n      - \"8080:80\"\n";
    let update_body = UpdateStackBody {
        stack_file_content: Some(updated_compose.to_string()),
        env: None,
        prune: None,
        pull_image: None,
        rollback_to: None,
    };
    instance
        .client
        .update_stack(stack_id, eid, &update_body)
        .await
        .expect("update_stack failed");

    let file_after = instance
        .client
        .get_stack_file(stack_id, None, None)
        .await
        .expect("get_stack_file after update failed");
    assert!(
        file_after.stack_file_content.contains("8080:80"),
        "expected updated port mapping"
    );

    // 6. Stop stack
    let stopped = instance
        .client
        .stop_stack(stack_id, eid)
        .await
        .expect("stop_stack failed");
    assert_eq!(stopped.status, 2, "expected status 2 (inactive)");

    // 7. Start stack
    let started = instance
        .client
        .start_stack(stack_id, eid)
        .await
        .expect("start_stack failed");
    assert_eq!(started.status, 1, "expected status 1 (active)");

    // 8. List with filter
    let filter = format!(r#"{{"EndpointID":{}}}"#, eid);
    let filtered = instance
        .client
        .list_stacks(Some(&filter))
        .await
        .expect("list_stacks with filter failed");
    assert_eq!(filtered.len(), 1);

    // 9. Delete stack
    instance
        .client
        .delete_stack(stack_id, eid, Some(true))
        .await
        .expect("delete_stack failed");

    let stacks_after = instance
        .client
        .list_stacks(None)
        .await
        .expect("list_stacks after delete failed");
    assert!(stacks_after.is_empty(), "expected no stacks after delete");
}
