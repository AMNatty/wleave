use crate::button::{WButtonAction, WButtonActionHandler};
use std::process::Command;
use tracing::error;

pub fn run_command(command: WButtonAction) {
    match command.handler {
        WButtonActionHandler::Executable(exe) => {
            if let Err(e) = Command::new(exe).spawn() {
                error!("Execution error: {e}");
            }
        }
        WButtonActionHandler::Shell(script) => {
            if let Err(e) = Command::new("sh").args(["-c", &script]).spawn() {
                error!("Execution error: {e}");
            }
        }
    }
}
