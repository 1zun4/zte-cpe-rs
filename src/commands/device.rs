use serde::Serialize;
use super::GoformCommand;

#[derive(Serialize, Default)]
pub struct RebootCommand { }

impl GoformCommand for RebootCommand {
    fn goform_id(&self) -> &'static str {
        "REBOOT_DEVICE"
    }
    
    fn authenticated(&self) -> bool {
        true
    }
}