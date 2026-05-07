use crate::{error::{BrowserError, Result},
            tools::{Tool, ToolContext, ToolResult}};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputParams {
    /// CSS selector (use either this or index, not both)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// Element index from DOM tree (use either this or selector, not both)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    /// Text to type into the element
    pub text: String,

    /// Clear existing content first (default: false)
    #[serde(default)]
    pub clear: bool,
}

#[derive(Default)]
pub struct InputTool;

impl Tool for InputTool {
    type Params = InputParams;

    fn name(&self) -> &str {
        "input"
    }

    fn execute_typed(&self, params: InputParams, context: &mut ToolContext) -> Result<ToolResult> {
        // Validate that exactly one selector method is provided
        match (&params.selector, &params.index) {
            (Some(_), Some(_)) => {
                return Err(BrowserError::ToolExecutionFailed {
                    tool: "input".to_string(),
                    reason: "Cannot specify both 'selector' and 'index'. Use one or the other.".to_string(),
                });
            }
            (None, None) => {
                return Err(BrowserError::ToolExecutionFailed {
                    tool: "input".to_string(),
                    reason: "Must specify either 'selector' or 'index'.".to_string(),
                });
            }
            _ => {}
        }

        // Get the CSS selector (either directly or from index)
        let css_selector = if let Some(selector) = params.selector.clone() {
            selector
        } else if let Some(index) = params.index {
            let dom = context.get_dom()?;
            let selector = dom
                .get_selector(index)
                .ok_or_else(|| BrowserError::ElementNotFound(format!("No element with index {}", index)))?;
            selector.clone()
        } else {
            unreachable!("Validation above ensures one field is Some")
        };

        let selector_json = serde_json::to_string(&css_selector).map_err(|e| BrowserError::ToolExecutionFailed {
            tool: "input".to_string(),
            reason: format!("Failed to encode selector: {}", e),
        })?;
        let text_json = serde_json::to_string(&params.text).map_err(|e| BrowserError::ToolExecutionFailed {
            tool: "input".to_string(),
            reason: format!("Failed to encode text: {}", e),
        })?;

        let script = format!(
            r#"(() => {{
                const selector = {selector_json};
                const text = {text_json};
                const clear = {clear};
                const el = document.querySelector(selector);

                if (!el) {{
                    return JSON.stringify({{ ok: false, reason: `Element not found: ${{selector}}` }});
                }}

                const target = el.matches('input, textarea, [contenteditable="true"], [contenteditable=""]')
                    ? el
                    : el.querySelector('input, textarea, [contenteditable="true"], [contenteditable=""]');

                if (!target) {{
                    return JSON.stringify({{ ok: false, reason: `Element is not fillable: ${{selector}}` }});
                }}

                target.scrollIntoView({{ block: 'center', inline: 'nearest' }});
                target.focus();

                const tag = target.tagName.toLowerCase();
                const isTextControl = tag === 'input' || tag === 'textarea';
                const nextValue = clear ? text : ((isTextControl ? target.value : target.textContent) || '') + text;

                if (isTextControl) {{
                    const proto = tag === 'textarea' ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
                    const descriptor = Object.getOwnPropertyDescriptor(proto, 'value');
                    if (descriptor && descriptor.set) {{
                        descriptor.set.call(target, nextValue);
                    }} else {{
                        target.value = nextValue;
                    }}
                }} else {{
                    target.textContent = nextValue;
                }}

                target.dispatchEvent(new InputEvent('input', {{
                    bubbles: true,
                    cancelable: true,
                    inputType: clear ? 'insertReplacementText' : 'insertText',
                    data: text,
                }}));
                target.dispatchEvent(new Event('change', {{ bubbles: true }}));

                return JSON.stringify({{
                    ok: true,
                    selector,
                    tag,
                    value: isTextControl ? target.value : target.textContent,
                }});
            }})()"#,
            selector_json = selector_json,
            text_json = text_json,
            clear = params.clear
        );

        let raw = context
            .session
            .tab()?
            .evaluate(&script, false)
            .map_err(|e| BrowserError::ToolExecutionFailed { tool: "input".to_string(), reason: e.to_string() })?;

        let data = raw
            .value
            .and_then(|value| value.as_str().map(str::to_owned))
            .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
            .unwrap_or_else(|| serde_json::json!({ "ok": false, "reason": "Input script did not return a value" }));

        if data.get("ok").and_then(|value| value.as_bool()) == Some(true) {
            Ok(ToolResult::success_with(data))
        } else {
            Err(BrowserError::ToolExecutionFailed {
                tool: "input".to_string(),
                reason: data
                    .get("reason")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Failed to fill input")
                    .to_string(),
            })
        }
    }
}
