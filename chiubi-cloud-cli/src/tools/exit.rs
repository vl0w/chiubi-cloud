use super::{ToolDescription, ToolError, ToolResult};

pub const TOOL: ToolDescription = ToolDescription {
    name: "exit",
    description: "Exit program",
    execute_interactive: execute,
    is_active: || true,
};

fn execute() -> ToolResult {
    Err(ToolError::Abort)
}
