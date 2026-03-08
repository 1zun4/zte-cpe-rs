use super::UbusCommand;
use serde::Serialize;

/// Get SMS settings (center number, validity, delivery report).
#[derive(Serialize, Default)]
pub struct GetSmsParameterCommand {}

impl UbusCommand for GetSmsParameterCommand {
    fn module(&self) -> &'static str {
        "zwrt_wms"
    }
    fn method(&self) -> &'static str {
        "zte_wms_get_parameter"
    }
}
