//! Request handlers for the SHU MCP Server.

use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::ServerHandler,
    schema::{
        CallToolRequestParams, CallToolResult, ListResourcesResult, ListToolsResult,
        PaginatedRequestParams, ReadResourceContent, ReadResourceRequestParams, ReadResourceResult,
        Resource, RpcError, TextResourceContents, schema_utils::CallToolError,
    },
};
use std::sync::Arc;

/// A custom handler implementing the `ServerHandler` trait for this server.
#[derive(Debug)]
pub struct MyServerHandler;

#[async_trait]
impl ServerHandler for MyServerHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: vec![], // Add your tools here
        })
    }

    async fn handle_list_resources_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListResourcesResult, RpcError> {
        Ok(ListResourcesResult {
            meta: None,
            next_cursor: None,
            resources: vec![Resource {
                uri: "shu://example".into(),
                name: "Example Resource".into(),
                description: Some("An example resource".into()),
                mime_type: Some("text/plain".into()),
                annotations: None,
                icons: vec![],
                meta: None,
                size: None,
                title: None,
            }],
        })
    }

    async fn handle_read_resource_request(
        &self,
        params: ReadResourceRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ReadResourceResult, RpcError> {
        if params.uri == "shu://example" {
            Ok(ReadResourceResult {
                meta: None,
                contents: vec![ReadResourceContent::TextResourceContents(
                    TextResourceContents {
                        uri: params.uri,
                        mime_type: Some("text/plain".into()),
                        text: "This is an example resource content.".into(),
                        meta: None,
                    },
                )],
            })
        } else {
            Err(RpcError::invalid_params()
                .with_message(format!("Unknown resource URI: {}", params.uri)))
        }
    }

    async fn handle_call_tool_request(
        &self,
        _params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        Err(CallToolError::new(RpcError::method_not_found().with_message("No tools implemented".into())))
    }
}
