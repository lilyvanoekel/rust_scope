use nih_plug::prelude::*;

use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Params)]
pub struct PluginParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "scale"]
    pub scale: FloatParam,
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(600, 400),
            scale: FloatParam::new(
                "Scale",
                0.0,
                FloatRange::Linear {
                    min: -0.0,
                    max: 1.0,
                },
            )
            .with_step_size(0.01),
        }
    }
}
