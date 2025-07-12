use nih_plug::params::smoothing::AtomicF32;
use nih_plug_egui::egui::{self};
use std::sync::atomic::Ordering;

pub fn oscilloscope(
    ui: &mut egui::Ui,
    rect: &egui::Rect,
    timebase: f32,
    vertical_scale: f32,
    current_buffer_size: usize,
    current_write_pos: usize,
    current_sample_rate: f32,
    buffer: &[AtomicF32],
    on_timebase_change: impl Fn(f32),
    on_scale_change: impl Fn(f32),
) {
    let ui_color = egui::Color32::from_rgb(200, 200, 200);

    let relative_unit = rect.width() * 0.01;
    let padding = relative_unit * 1.2;
    let font_size = relative_unit * 2.0;

    // Draw param labels
    ui.painter().text(
        egui::pos2(rect.left() + padding, rect.top() + padding),
        egui::Align2::LEFT_TOP,
        &format!("Timebase: {:.1} ms", timebase),
        egui::FontId::proportional(font_size),
        ui_color,
    );
    ui.painter().text(
        egui::pos2(
            rect.left() + padding,
            rect.top() + padding + font_size * 1.6,
        ),
        egui::Align2::LEFT_TOP,
        &format!("Scale: {:.1}x", vertical_scale),
        egui::FontId::proportional(font_size),
        ui_color,
    );

    if current_buffer_size > 1 {
        let display_width = rect.width() as usize;

        // Calculate how many samples to display based on timebase
        let samples_to_display = (current_sample_rate * timebase / 1000.0) as usize;
        let samples_to_display = samples_to_display.min(current_buffer_size);

        // Calculate the points we want to draw
        let points: Vec<egui::Pos2> = (0..display_width)
            .map(|i| {
                // Map screen position to buffer position, using timebase
                let buffer_index = (current_write_pos + (i * samples_to_display / display_width))
                    % current_buffer_size;
                let sample = buffer[buffer_index].load(Ordering::Relaxed);
                let x = rect.left() + i as f32;
                let y = rect.center().y - sample * rect.height() * 0.4 * vertical_scale;
                egui::pos2(x, y)
            })
            .collect();

        // Draw middle line
        ui.painter().add(egui::Shape::line(
            vec![
                egui::pos2(rect.left(), rect.center().y),
                egui::pos2(rect.right(), rect.center().y),
            ],
            egui::Stroke::new(1.0, ui_color),
        ));

        // Draw oscilloscope lines themselves
        ui.painter().add(egui::Shape::line(
            points,
            egui::Stroke::new(1.0, egui::Color32::LIGHT_GREEN),
        ));
    }

    if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
        if rect.contains(pointer_pos) {
            if ui.input(|i| i.pointer.primary_down()) {
                // Check if this is the start of a drag
                if ui.input(|i| i.pointer.primary_pressed()) {
                    // Store initial position and values when drag starts
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(
                            ui.id().with("drag_start"),
                            (pointer_pos.x, pointer_pos.y, timebase, vertical_scale),
                        )
                    });
                } else {
                    // We're in the middle of a drag - calculate relative movement
                    if let Some((start_x, start_y, start_timebase, start_scale)) =
                        ui.memory(|mem| {
                            mem.data
                                .get_temp::<(f32, f32, f32, f32)>(ui.id().with("drag_start"))
                        })
                    {
                        let horizontal_movement = pointer_pos.x - start_x;
                        let vertical_movement = start_y - pointer_pos.y; // Inverted: up = more scale

                        // Horizontal drag controls timebase
                        let window_width = rect.width();
                        let timebase_sensitivity = 2.0;
                        let timebase_delta =
                            (horizontal_movement / window_width) * 99.0 * timebase_sensitivity;
                        let new_timebase = (start_timebase + timebase_delta).clamp(1.0, 100.0);
                        on_timebase_change(new_timebase);

                        // Vertical drag controls scale
                        let window_height = rect.height();
                        let scale_sensitivity = 1.0;
                        let scale_delta =
                            (vertical_movement / window_height) * 9.5 * scale_sensitivity;
                        let new_scale = (start_scale + scale_delta).clamp(0.5, 10.0);
                        on_scale_change(new_scale);
                    }
                }
            }
        }
    }
}
