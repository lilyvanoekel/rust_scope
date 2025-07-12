use nih_plug::params::smoothing::AtomicF32;
use nih_plug_egui::egui::{self, Response, Sense, Widget};
use std::sync::atomic::Ordering;

pub struct OscilloscopeWidget<'a> {
    timebase: f32,
    vertical_scale: f32,
    current_buffer_size: usize,
    current_write_pos: usize,
    current_sample_rate: f32,
    buffer: &'a [AtomicF32],
    on_timebase_change: Option<Box<dyn Fn(f32) + 'a>>,
    on_scale_change: Option<Box<dyn Fn(f32) + 'a>>,
}

impl<'a> OscilloscopeWidget<'a> {
    pub fn new(
        timebase: f32,
        vertical_scale: f32,
        current_buffer_size: usize,
        current_write_pos: usize,
        current_sample_rate: f32,
        buffer: &'a [AtomicF32],
    ) -> Self {
        Self {
            timebase,
            vertical_scale,
            current_buffer_size,
            current_write_pos,
            current_sample_rate,
            buffer,
            on_timebase_change: None,
            on_scale_change: None,
        }
    }

    pub fn on_timebase_change(mut self, callback: impl Fn(f32) + 'a) -> Self {
        self.on_timebase_change = Some(Box::new(callback));
        self
    }

    pub fn on_scale_change(mut self, callback: impl Fn(f32) + 'a) -> Self {
        self.on_scale_change = Some(Box::new(callback));
        self
    }
}

impl<'a> Widget for OscilloscopeWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        // Take up all space available
        let desired_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());

        // Attempt to set cursor, shows intent but doesn't actually work at this point (@todo)
        ui.ctx().set_cursor_icon(egui::CursorIcon::Move);

        // Color for UI elements (centre line, text)
        let ui_color = egui::Color32::from_rgb(200, 200, 200);
        let scope_color = egui::Color32::LIGHT_GREEN;

        // Relative unit system based on width, something more sophisticated is probably warranted
        // This works ok when a single full size instance of the widget is used, but less so in a UI composed of multiple widgets
        let relative_unit = rect.width() * 0.01;
        let padding = relative_unit * 1.2;
        let font_size = relative_unit * 2.0;

        // Draw param labels
        ui.painter().text(
            egui::pos2(rect.left() + padding, rect.top() + padding),
            egui::Align2::LEFT_TOP,
            &format!("Timebase: {:.1} ms", self.timebase),
            egui::FontId::proportional(font_size),
            ui_color,
        );
        ui.painter().text(
            egui::pos2(
                rect.left() + padding,
                rect.top() + padding + font_size * 1.6,
            ),
            egui::Align2::LEFT_TOP,
            &format!("Scale: {:.1}x", self.vertical_scale),
            egui::FontId::proportional(font_size),
            ui_color,
        );

        // Draw oscilloscope
        if self.current_buffer_size > 1 {
            let display_width = rect.width() as usize;

            // Calculate how many samples to display based on timebase
            let samples_to_display = (self.current_sample_rate * self.timebase / 1000.0) as usize;
            let samples_to_display = samples_to_display.min(self.current_buffer_size);

            // Calculate the points we want to draw
            let points: Vec<egui::Pos2> = (0..display_width)
                .map(|i| {
                    // Map screen position to buffer position, using timebase
                    let buffer_index = (self.current_write_pos
                        + (i * samples_to_display / display_width))
                        % self.current_buffer_size;

                    let sample = self.buffer[buffer_index].load(Ordering::Relaxed);
                    let x = rect.left() + i as f32;
                    let y = rect.center().y - sample * rect.height() * 0.4 * self.vertical_scale;
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
                egui::Stroke::new(1.0, scope_color),
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
                                (
                                    pointer_pos.x,
                                    pointer_pos.y,
                                    self.timebase,
                                    self.vertical_scale,
                                ),
                            )
                        });
                    } else {
                        // We're in the middle of a drag, calculate relative movement
                        if let Some((start_x, start_y, start_timebase, start_scale)) =
                            ui.memory(|mem| {
                                mem.data
                                    .get_temp::<(f32, f32, f32, f32)>(ui.id().with("drag_start"))
                            })
                        {
                            let horizontal_movement = pointer_pos.x - start_x;
                            let vertical_movement = start_y - pointer_pos.y; // up = more scale

                            // Horizontal drag controls timebase
                            let window_width = rect.width();
                            let timebase_sensitivity = 2.0;
                            let timebase_delta =
                                (horizontal_movement / window_width) * 99.0 * timebase_sensitivity;
                            let new_timebase = (start_timebase + timebase_delta).clamp(1.0, 100.0);
                            if let Some(ref callback) = self.on_timebase_change {
                                callback(new_timebase);
                            }

                            // Vertical drag controls scale
                            let window_height = rect.height();
                            let scale_sensitivity = 1.0;
                            let scale_delta =
                                (vertical_movement / window_height) * 9.5 * scale_sensitivity;
                            let new_scale = (start_scale + scale_delta).clamp(0.5, 10.0);
                            if let Some(ref callback) = self.on_scale_change {
                                callback(new_scale);
                            }
                        }
                    }
                }
            }
        }

        response
    }
}
