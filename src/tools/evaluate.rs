use crate::{error::{BrowserError, Result},
            tools::{Tool, ToolContext, ToolResult}};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvaluateParams {
    /// JavaScript code to execute
    pub code: String,

    /// Wait for promise resolution (default: false)
    #[serde(default)]
    pub await_promise: bool,
}

#[derive(Default)]
pub struct EvaluateTool;

impl Tool for EvaluateTool {
    type Params = EvaluateParams;

    fn name(&self) -> &str {
        "evaluate"
    }

    fn execute_typed(&self, params: EvaluateParams, context: &mut ToolContext) -> Result<ToolResult> {
        let wrapped_code = format!(
            r#"(async () => {{
                const __browserUseEncode = (value) => {{
                    if (typeof value === 'undefined') {{
                        return JSON.stringify({{ type: 'undefined' }});
                    }}
                    try {{
                        return JSON.stringify({{ type: 'value', value }});
                    }} catch (error) {{
                        return JSON.stringify({{
                            type: 'unserializable',
                            value: Object.prototype.toString.call(value),
                            error: String(error && error.message || error),
                        }});
                    }}
                }};

                try {{
                    const value = await ({code});
                    return __browserUseEncode(value);
                }} catch (error) {{
                    return JSON.stringify({{
                        type: 'error',
                        error: String(error && (error.stack || error.message) || error),
                    }});
                }}
            }})()"#,
            code = params.code
        );

        let result = context
            .session
            .tab()?
            .evaluate(&wrapped_code, true)
            .map_err(|e| BrowserError::EvaluationFailed(e.to_string()))?;

        let result_value = match result.value {
            Some(Value::String(serialized)) => {
                let decoded: Value = serde_json::from_str(&serialized).unwrap_or(Value::Null);
                match decoded.get("type").and_then(|value| value.as_str()) {
                    Some("value") => decoded.get("value").cloned().unwrap_or(Value::Null),
                    Some("undefined") => Value::Null,
                    Some("unserializable") => decoded,
                    Some("error") => {
                        return Err(BrowserError::EvaluationFailed(
                            decoded
                                .get("error")
                                .and_then(|value| value.as_str())
                                .unwrap_or("JavaScript evaluation failed")
                                .to_string(),
                        ));
                    }
                    _ => Value::String(serialized),
                }
            }
            Some(value) => value,
            None => Value::Null,
        };

        Ok(ToolResult::success_with(serde_json::json!({
            "result": result_value,
            "awaited": params.await_promise
        })))
    }
}
