//! MCP (Model Context Protocol) server implementation for browser automation.

pub mod handler;
pub use handler::BrowserServer;

use crate::tools::{self, Tool, ToolContext, ToolResult as InternalToolResult};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router, ErrorData as McpError,
};

fn convert_result(result: InternalToolResult) -> Result<CallToolResult, McpError> {
    if result.success {
        let text = if let Some(data) = result.data {
            serde_json::to_string_pretty(&data).unwrap_or_else(|_| data.to_string())
        } else {
            "Success".to_string()
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    } else {
        Err(McpError::internal_error(
            result.error.unwrap_or_else(|| "Unknown error".to_string()),
            None,
        ))
    }
}

macro_rules! register_mcp_tools {
    ($($mcp_name:ident => $tool_type:ty, $description:expr);* $(;)?) => {
        #[tool_router]
        impl BrowserServer {
            #[tool(description = "Close the browser when the task is complete")]
            fn browser_close(&self, _params: Parameters<tools::close::CloseParams>) -> Result<CallToolResult, McpError> {
                self.close_session()?;
                convert_result(InternalToolResult::success_with(serde_json::json!({
                    "message": "Browser closed successfully"
                })))
            }

            $(
                #[tool(description = $description)]
                fn $mcp_name(
                    &self,
                    params: Parameters<<$tool_type as Tool>::Params>,
                ) -> Result<CallToolResult, McpError> {
                    self.with_session(|session| {
                        let mut context = ToolContext::new(session);
                        let tool = <$tool_type>::default();
                        let result = tool.execute_typed(params.0, &mut context)
                            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                        convert_result(result)
                    })
                }
            )*
        }
    };
}

register_mcp_tools! {
    browser_open => tools::open_browser::OpenBrowserTool, "Open or connect the browser on demand, optionally navigating to a URL";
    browser_navigate => tools::navigate::NavigateTool, "Navigate to a specified URL in the browser";
    browser_go_back => tools::go_back::GoBackTool, "Navigate back in browser history";
    browser_go_forward => tools::go_forward::GoForwardTool, "Navigate forward in browser history";
    browser_wait => tools::wait::WaitTool, "Wait for an element to appear on the page";

    browser_snapshot => tools::snapshot::SnapshotTool, "Get a snapshot of the current page with indexed interactive elements";
    browser_extract => tools::extract::ExtractContentTool, "Extract text or HTML content from the page or an element";
    browser_get_markdown => tools::markdown::GetMarkdownTool, "Get markdown content from the current page";
    browser_read_links => tools::read_links::ReadLinksTool, "Read all links on the current page";
    browser_screenshot => tools::screenshot::ScreenshotTool, "Capture a screenshot of the current page";
    browser_annotate => tools::annotate::AnnotateTool, "Capture a screenshot annotated with interactive element indexes";
    browser_evaluate => tools::evaluate::EvaluateTool, "Execute JavaScript code in the browser context";

    browser_click => tools::click::ClickTool, "Click an element by CSS selector or snapshot index";
    browser_hover => tools::hover::HoverTool, "Hover an element by CSS selector or snapshot index";
    browser_select => tools::select::SelectTool, "Select an option in a dropdown by CSS selector or snapshot index";
    browser_input_fill => tools::input::InputTool, "Type text into an input element by CSS selector or snapshot index";
    browser_press_key => tools::press_key::PressKeyTool, "Press a keyboard key";
    browser_scroll => tools::scroll::ScrollTool, "Scroll the page by a specified amount or to the bottom";

    browser_new_tab => tools::new_tab::NewTabTool, "Open a new tab and navigate to a URL";
    browser_tab_list => tools::tab_list::TabListTool, "List all browser tabs";
    browser_switch_tab => tools::switch_tab::SwitchTabTool, "Switch to a tab by index";
    browser_close_tab => tools::close_tab::CloseTabTool, "Close the active tab";

    browser_get_cookies => tools::cookies::GetCookiesTool, "Get cookies from the current browser session";
    browser_set_cookies => tools::cookies::SetCookiesTool, "Set cookies in the current browser session";
    browser_get_local_storage => tools::local_storage::GetLocalStorageTool, "Get localStorage from the current page";
    browser_set_local_storage => tools::local_storage::SetLocalStorageTool, "Set a localStorage item on the current page";
    browser_remove_local_storage => tools::local_storage::RemoveLocalStorageTool, "Remove a localStorage item from the current page";
    browser_clear_local_storage => tools::local_storage::ClearLocalStorageTool, "Clear localStorage on the current page";

    browser_get_console_logs => tools::debug::GetConsoleLogsTool, "Get captured console logs";
    browser_get_network_errors => tools::debug::GetNetworkErrorsTool, "Get captured network errors";

    browser_sitemap => tools::sitemap::SitemapTool, "Analyze sitemap URLs and optional page structure for a site";
}
