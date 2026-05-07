use crate::{
    error::Result,
    tools::{Tool, ToolContext, ToolResult},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct OpenBrowserParams {
    /// Optional URL to navigate to after opening the browser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Default)]
pub struct OpenBrowserTool;

impl Tool for OpenBrowserTool {
    type Params = OpenBrowserParams;

    fn name(&self) -> &str {
        "open_browser"
    }

    fn execute_typed(&self, params: OpenBrowserParams, context: &mut ToolContext) -> Result<ToolResult> {
        if let Some(url) = params.url {
            context.session.navigate(&url)?;
            context.session.wait_for_navigation()?;
        }

        let tab = context.session.tab()?;
        Ok(ToolResult::success_with(serde_json::json!({
            "opened": true,
            "title": tab.get_title().unwrap_or_default(),
            "url": tab.get_url()
        })))
    }
}
