use nih_plug::params::smoothing::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self},
    resizable_window::ResizableWindow,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

mod params;

use params::PluginParams;

// Maximum supported sample rate (192k)
const MAX_SAMPLE_RATE: f32 = 192_000.0;

// Buffer size for 100ms at maximum sample rate
const MAX_BUFFER_SIZE: usize = (MAX_SAMPLE_RATE * 0.1) as usize;

struct MyPlugin {
    params: Arc<PluginParams>,
    buffer: Arc<[AtomicF32; MAX_BUFFER_SIZE]>,
    write_pos: Arc<AtomicUsize>,
    sample_rate: Arc<AtomicU32>,
    buffer_size: Arc<AtomicUsize>,
}

impl Default for MyPlugin {
    fn default() -> Self {
        let buffer = std::array::from_fn(|_| AtomicF32::new(0.0));
        Self {
            params: Arc::new(PluginParams::default()),
            buffer: Arc::new(buffer),
            write_pos: Arc::new(AtomicUsize::new(0)),
            sample_rate: Arc::new(AtomicU32::new(44_100)),
            buffer_size: Arc::new(AtomicUsize::new(4410)), // Default 100ms at 44.1kHz
        }
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "Scope";
    const VENDOR: &'static str = "Lily's Nonexistent Company";
    const URL: &'static str = "https://lilyvanoekel.com";
    const EMAIL: &'static str = "why@doyouneed.this";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate;
        if sample_rate > MAX_SAMPLE_RATE {
            return false;
        }

        // Calculate buffer size for 100ms at current sample rate
        let buffer_size = (sample_rate as f32 * 0.1) as usize;
        self.buffer_size.store(buffer_size, Ordering::Relaxed);
        self.sample_rate
            .store(sample_rate as u32, Ordering::Relaxed);
        self.write_pos.store(0, Ordering::Relaxed);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let buffer_size = self.buffer_size.load(Ordering::Relaxed);
        let channel_count = buffer.channels();
        let division_factor = 1.0 / (channel_count as f32);

        for channel_samples in buffer.iter_samples() {
            // Average out samples for all channels
            let mut sum = 0.0;
            for sample in channel_samples {
                sum += *sample;
            }
            let mixed_sample = sum * division_factor;

            // Store the sample in the buffer
            let write_pos = self.write_pos.load(Ordering::Relaxed);
            self.buffer[write_pos].store(mixed_sample, Ordering::Relaxed);
            self.write_pos
                .store((write_pos + 1) % buffer_size, Ordering::Relaxed);
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        self.write_pos.store(0, Ordering::Relaxed);
    }

    fn deactivate(&mut self) {}

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let buffer = self.buffer.clone();
        let write_pos = self.write_pos.clone();
        let buffer_size = self.buffer_size.clone();
        let egui_state = self.params.editor_state.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, _setter, _state| {
                ResizableWindow::new("Oscilloscope").show(egui_ctx, egui_state.as_ref(), |ui| {
                    let rect = ui.available_rect_before_wrap();

                    let current_buffer_size = buffer_size.load(Ordering::Relaxed);
                    let current_write_pos = write_pos.load(Ordering::Relaxed);

                    if current_buffer_size > 1 {
                        let display_width = rect.width() as usize;

                        // Calculate the points we want to draw
                        let points: Vec<egui::Pos2> = (0..display_width)
                            .map(|i| {
                                // Map screen position to buffer position
                                let buffer_index = (current_write_pos
                                    + (i * current_buffer_size / display_width))
                                    % current_buffer_size;
                                let sample = buffer[buffer_index].load(Ordering::Relaxed);
                                let x = rect.left() + i as f32;
                                let y = rect.center().y - sample * rect.height() * 0.4;
                                egui::pos2(x, y)
                            })
                            .collect();

                        // Draw middle line
                        ui.painter().add(egui::Shape::line(
                            vec![
                                egui::pos2(rect.left(), rect.center().y),
                                egui::pos2(rect.right(), rect.center().y),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)),
                        ));

                        // Draw oscilloscope
                        ui.painter().add(egui::Shape::line(
                            points,
                            egui::Stroke::new(1.0, egui::Color32::LIGHT_GREEN),
                        ));
                    }

                    ui.allocate_rect(rect, egui::Sense::hover());
                });
            },
        )
    }
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.lilyvanoekel.scope";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Scope");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

nih_export_clap!(MyPlugin);
