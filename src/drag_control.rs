use nih_plug_egui::egui;

// This trait allows the user of a widget to modify params by dragging the mouse up/down over the widget
pub trait DragControllable {
    fn on_drag(&mut self, normalized_x: f32, normalized_y: f32);
    fn initial_drag_position(&self) -> (f32, f32) {
        (0.5, 0.5)
    }

    fn handle_drag_control(&mut self, ui: &mut egui::Ui, rect: &egui::Rect) {
        let id = ui.id().with("drag_state");
        if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if rect.contains(pointer_pos) {
                if ui.input(|i| i.pointer.primary_down()) {
                    if ui.input(|i| i.pointer.primary_pressed()) {
                        // Drag start: store initial normalized position
                        let (init_x, init_y) = self.initial_drag_position();
                        ui.memory_mut(|mem| {
                            mem.data
                                .insert_temp(id, (pointer_pos.x, pointer_pos.y, init_x, init_y));
                        });
                    } else if let Some((start_x, start_y, start_norm_x, start_norm_y)) =
                        ui.memory(|mem| mem.data.get_temp::<(f32, f32, f32, f32)>(id))
                    {
                        // Calculate normalized deltas
                        let delta_x = (pointer_pos.x - start_x) / rect.width();
                        let delta_y = (start_y - pointer_pos.y) / rect.height();
                        // Update normalized position, clamp to 0..1
                        let new_norm_x = (start_norm_x + delta_x).clamp(0.0, 1.0);
                        let new_norm_y = (start_norm_y + delta_y).clamp(0.0, 1.0);
                        // Call widget's on_drag
                        self.on_drag(new_norm_x, new_norm_y);
                    }
                }
            }
        }
    }
}
