use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging must go to stderr — stdout is the MCP transport.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Portainer MCP server");

    let service = portainer_mcp::server::PortainerServer::new()
        .serve(stdio())
        .await?;
    service.waiting().await?;

    Ok(())
}
