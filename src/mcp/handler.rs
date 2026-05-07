//! ServerHandler implementation for BrowserSession

use crate::browser::{BrowserSession, ConnectionOptions, LaunchOptions};
use log::debug;
use rmcp::{ServerHandler,
           handler::server::tool::ToolRouter,
           model::{ServerCapabilities, ServerInfo},
           tool_handler};
use rmcp::ErrorData as McpError;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
enum BrowserStartup {
    Launch(LaunchOptions),
    Connect(ConnectionOptions),
}

/// MCP Server wrapper for BrowserSession
///
/// This struct holds a browser session and provides thread-safe access
/// for MCP tool execution.
#[derive(Clone)]
pub struct BrowserServer {
    session: Arc<Mutex<Option<BrowserSession>>>,
    startup: BrowserStartup,
    tool_router: ToolRouter<Self>,
}

impl BrowserServer {
    /// Create a new browser server with default launch options
    pub fn new() -> Result<Self, String> {
        Self::with_options(LaunchOptions::default())
    }

    /// Create a new browser server with custom launch options
    pub fn with_options(options: LaunchOptions) -> Result<Self, String> {
        Ok(Self {
            session: Arc::new(Mutex::new(None)),
            startup: BrowserStartup::Launch(options),
            tool_router: Self::tool_router(),
        })
    }

    /// Create a browser server connected to an existing Chrome DevTools WebSocket endpoint.
    pub fn with_connection(options: ConnectionOptions) -> Result<Self, String> {
        Ok(Self {
            session: Arc::new(Mutex::new(None)),
            startup: BrowserStartup::Connect(options),
            tool_router: Self::tool_router(),
        })
    }

    /// Run a function with a lazily-created browser session.
    pub(crate) fn with_session<T>(
        &self,
        f: impl FnOnce(&BrowserSession) -> Result<T, McpError>,
    ) -> Result<T, McpError> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| McpError::internal_error("Failed to lock browser session", None))?;

        if session.is_none() {
            let created = match &self.startup {
                BrowserStartup::Launch(options) => BrowserSession::launch(options.clone())
                    .map_err(|e| McpError::internal_error(format!("Failed to launch browser: {}", e), None))?,
                BrowserStartup::Connect(options) => BrowserSession::connect(options.clone())
                    .map_err(|e| McpError::internal_error(format!("Failed to connect browser: {}", e), None))?,
            };
            *session = Some(created);
        }

        f(session.as_ref().expect("session initialized above"))
    }

    /// Close the current browser session and clear it so the next browser_open
    /// call starts a fresh browser instance.
    pub(crate) fn close_session(&self) -> Result<(), McpError> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| McpError::internal_error("Failed to lock browser session", None))?;

        if let Some(existing) = session.take() {
            existing
                .close()
                .map_err(|e| McpError::internal_error(format!("Failed to close browser: {}", e), None))?;
        }

        Ok(())
    }
}

impl Default for BrowserServer {
    fn default() -> Self {
        Self::new().expect("Failed to create default browser server")
    }
}

impl Drop for BrowserServer {
    fn drop(&mut self) {
        debug!("BrowserServer dropped");
    }
}

#[tool_handler]
impl ServerHandler for BrowserServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Browser-use MCP Server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
