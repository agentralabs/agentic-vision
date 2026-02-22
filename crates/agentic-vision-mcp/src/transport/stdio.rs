//! Stdio transport — reads JSON-RPC from stdin, writes to stdout.

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use crate::protocol::ProtocolHandler;
use crate::types::{JsonRpcError, McpError, McpResult, RequestId, JSONRPC_VERSION};

use super::framing;

/// Stdio transport for desktop MCP clients.
pub struct StdioTransport {
    handler: ProtocolHandler,
}

impl StdioTransport {
    pub fn new(handler: ProtocolHandler) -> Self {
        Self { handler }
    }

    /// Run the transport loop — reads from stdin, writes to stdout.
    pub async fn run(&self) -> McpResult<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        let mut content_length: Option<usize> = None;
        let mut framed_output = false;

        tracing::info!("Stdio transport started");

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await.map_err(McpError::Io)?;

            if bytes_read == 0 {
                tracing::info!("EOF on stdin, shutting down");
                break;
            }

            let trimmed = line.trim_end_matches(['\r', '\n']);

            let lower = trimmed.to_ascii_lowercase();
            if lower.starts_with("content-length:") {
                let rest = trimmed.split_once(':').map(|(_, rhs)| rhs).unwrap_or("");
                content_length = rest.trim().parse::<usize>().ok();
                if content_length.is_some() {
                    framed_output = true;
                }
                continue;
            }

            if let Some(n) = content_length {
                // Skip optional header separator line.
                if trimmed.is_empty() {
                    let mut body = vec![0u8; n];
                    reader.read_exact(&mut body).await.map_err(McpError::Io)?;
                    let payload = String::from_utf8_lossy(&body).to_string();

                    self.process_message(&payload, framed_output, &mut stdout)
                        .await?;
                    content_length = None;
                    continue;
                }

                // Ignore any remaining header lines (e.g. Content-Type).
                continue;
            }

            if trimmed.is_empty() {
                continue;
            }

            self.process_message(trimmed, framed_output, &mut stdout)
                .await?;
        }

        Ok(())
    }

    async fn process_message(
        &self,
        input: &str,
        framed_output: bool,
        stdout: &mut tokio::io::Stdout,
    ) -> McpResult<()> {
        match framing::parse_message(input.trim()) {
            Ok(msg) => {
                if let Some(response) = self.handler.handle_message(msg).await {
                    self.write_response(stdout, &response, framed_output)
                        .await?;
                }
            }
            Err(e) => {
                tracing::warn!("Parse error: {e}");
                let error_response = JsonRpcError {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    id: RequestId::Null,
                    error: crate::types::JsonRpcErrorObject {
                        code: e.code(),
                        message: e.to_string(),
                        data: None,
                    },
                };
                let value = serde_json::to_value(error_response)
                    .map_err(|err| McpError::InternalError(err.to_string()))?;
                self.write_response(stdout, &value, framed_output).await?;
            }
        }
        Ok(())
    }

    async fn write_response(
        &self,
        stdout: &mut tokio::io::Stdout,
        response: &serde_json::Value,
        framed_output: bool,
    ) -> McpResult<()> {
        if framed_output {
            let json = serde_json::to_string(response).map_err(McpError::Json)?;
            let header = format!("Content-Length: {}\r\n\r\n", json.len());
            stdout
                .write_all(header.as_bytes())
                .await
                .map_err(McpError::Io)?;
            stdout
                .write_all(json.as_bytes())
                .await
                .map_err(McpError::Io)?;
            stdout.flush().await.map_err(McpError::Io)?;
            return Ok(());
        }

        let framed = framing::frame_message(response)?;
        stdout
            .write_all(framed.as_bytes())
            .await
            .map_err(McpError::Io)?;
        stdout.flush().await.map_err(McpError::Io)?;
        Ok(())
    }
}
