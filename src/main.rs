use anyhow::Result;
use roberto_mcp::CodeAnalysisTools;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with stderr output
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting CodeCortext MCP Server");

    // Create and serve the server via stdio
    let service = CodeAnalysisTools::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
