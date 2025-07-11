use nih_plug::prelude::*;

use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Params)]
pub struct PluginParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "timebase"]
    pub timebase: FloatParam,
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(600, 400),
            timebase: FloatParam::new(
                "Timebase",
                10.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 100.0,
                },
            )
            .with_step_size(1.0)
            .with_unit("ms"),
        }
    }
}
