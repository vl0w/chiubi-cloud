use crate::tools::{ToolDescription, ToolError};

pub const TOOL: ToolDescription = ToolDescription {
    name: "print-config",
    description: "Prints your configuration",
    execute_interactive: print_config,
    is_active: super::is_config_existing,
};

fn print_config() -> Result<(), ToolError> {
    let config = super::read_config().ok_or(ToolError::NoPlexConfig)?;
    println!("{}", config);
    Ok(())
}
