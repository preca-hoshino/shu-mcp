//! SHU MCP Server
//!
//! This is the main entry point for the SHU MCP server, using `rust-mcp-sdk`.

pub mod handler;

use handler::MyServerHandler;
use rust_mcp_sdk::{
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
    error::SdkResult,
    mcp_server::{McpServerOptions, ServerRuntime, server_runtime},
    schema::{
        Implementation, InitializeResult, ProtocolVersion, ServerCapabilities,
        ServerCapabilitiesTools,
    },
};
use std::sync::Arc;

/// The main entry point to start the MCP server.
///
/// # Errors
/// Returns an error if the stdio transport fails to initialize or server fails to start.
#[tokio::main]
async fn main() -> SdkResult<()> {
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "shu-mcp".into(),
            version: "0.1.0".into(),
            title: Some("SHU MCP Server".into()),
            description: Some("A basic SHU MCP Server implementation".into()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            resources: Some(rust_mcp_sdk::schema::ServerCapabilitiesResources {
                subscribe: None,
                list_changed: None,
            }),
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some("Basic SHU MCP operations".into()),
        protocol_version: ProtocolVersion::V2025_11_25.into(),
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = MyServerHandler {};

    let server: Arc<ServerRuntime> = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
        message_observer: None,
    });

    if let Err(start_error) = server.start().await {
        eprintln!("{start_error}");
    }

    Ok(())
}
