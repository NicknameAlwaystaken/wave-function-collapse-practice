
mod wfc;

use std::time::{Duration, Instant};

use image::RgbImage;
use wfc::{collapse::collapse, engine::{WfcEngine, WfcEngineConfig}, util::get_available_tile_sets};

use eframe::egui::{self, Color32, ColorImage};

struct WfcApp {
    engine: WfcEngine,
    engine_config: WfcEngineConfig,
    is_playing: bool,
    last_step: Instant,
    slider_pos: f32,
    image_texture: Option<egui::TextureHandle>,
}

impl WfcApp {
    fn new(engine_config: WfcEngineConfig) -> Self {
        let engine = WfcEngine::new(engine_config.clone());

        Self {
            engine,
            engine_config,
            is_playing: false,
            last_step: Instant::now(),
            slider_pos: 1.0,
            image_texture: None,
        }
    }
}

fn rgbimage_to_colorimage(img: &RgbImage) -> ColorImage {
    let size = [img.width() as usize, img.height() as usize];

    let pixels = img
        .pixels()
        .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
        .collect();

    ColorImage { size, pixels }
}

impl eframe::App for WfcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();

        // logarithmic slider, such as easier to fine tune lower end near min ms
        let min = 1.0_f32;
        let max = 1000.0_f32;
        let value = min * (max / min).powf(self.slider_pos);
        let step_interval = Duration::from_millis(value.round() as u64);

        if self.is_playing && now.duration_since(self.last_step) >= step_interval {
            if !self.engine.solved {
                if let Ok(_) = collapse(&mut self.engine) {

                    let color_image = self.engine.draw();

                    self.image_texture =
                        Some(ctx.load_texture("preview", color_image, Default::default()));

                    ctx.request_repaint(); // schedule another frame to keep it running

                }
            } else {
                self.is_playing = false;
            }
            self.last_step = now;
        }

        egui::SidePanel::left("settings_panel").show(ctx, |ui| {
            ui.heading("WFC Settings");

            ui.label("Image size");
            ui.horizontal(|ui| {
                ui.label("Width:");
                let mut width_str = self.engine_config.width.to_string();
                ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut width_str));
                if let Ok(value) = width_str.parse::<usize>() {
                    self.engine_config.width = value;
                }

                ui.label("Height:");
                let mut height_str = self.engine_config.height.to_string();
                ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut height_str));
                if let Ok(value) = height_str.parse::<usize>() {
                    self.engine_config.height = value;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Tile Set Name:");
                egui::ComboBox::from_id_salt(&self.engine_config.tile_set_name)
                    .selected_text(format!("📁 {}", &self.engine_config.tile_set_name))
                    .show_ui(ui, |ui| {
                        for name in get_available_tile_sets(&self.engine_config.tile_path).iter() {
                            ui.selectable_value(&mut self.engine_config.tile_set_name, name.to_string(), name);
                        }
                    })
            });

            ui.horizontal(|ui| {
                ui.label("Tile Path:");
                ui.text_edit_singleline(&mut self.engine_config.tile_path);
            });

            let tile_size = self.engine_config.tile_size;
            ui.add(
                egui::Slider::new(&mut self.engine_config.tile_size, 1..=100)
                    .text(format!("Tile Size: {}", tile_size)),
            );

            if ui.button("Apply").clicked() {
                self.is_playing = false;
                self.engine = WfcEngine::new(self.engine_config.clone());
                let color_image = self.engine.draw();

                self.image_texture =
                    Some(ctx.load_texture("preview", color_image, Default::default()));
            }

        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Wave Function Collapse Application");

            ui.add(
                egui::Slider::new(&mut self.slider_pos, 0.0..=1.0)
                    .text(format!("Step every ({:.0} ms)", value)),
            );

            if ui.button("Reset").clicked() {
                self.is_playing = false;
                self.engine = WfcEngine::new(self.engine_config.clone());
                let color_image = self.engine.draw();

                self.image_texture =
                    Some(ctx.load_texture("preview", color_image, Default::default()));
            }

            if ui.button("Pause").clicked() {
                self.is_playing = false;
            }

            if ui.button("Start collapsing").clicked() {
                self.is_playing = true;
            }

            if ui.button("Collapse one").clicked() {
                // Temporarily borrow fields
                let mut engine = &mut self.engine;

                if !engine.solved {
                    match collapse(
                        &mut engine,
                    ) {
                        Ok(_) => {
                            let color_image = engine.draw();

                            self.image_texture =
                                Some(ctx.load_texture("preview", color_image, Default::default()));
                        }
                        Err(()) => {
                            println!("Collapse failed.");
                        }
                    }
                }
            }

            if let Some((collapsed, total)) = self.engine.progress {
                let fraction = collapsed as f32 / total as f32;

                // Show numeric count
                ui.label(format!("Collapsed: {} / {}", collapsed, total));

                // Show progress bar with percentage
                ui.add(egui::ProgressBar::new(fraction).show_percentage());
            } else {
                ui.label(format!("Collapsed: {} / {}", 0, 0));
                ui.add(egui::ProgressBar::new(0.0).show_percentage());
            }

            if let Some(tex) = &self.image_texture {
                ui.image(tex);
            }

        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 1000.0]),
        ..Default::default()
    };

    let tile_path = "main_wfc/src/assets/image/";
    let tile_set_name = "circuit";
    let tile_size = 14;

    let width = 100;
    let height = 50;

    let locks = vec![]; // or prefilled

    let config = WfcEngineConfig {
        tile_path: tile_path.to_string(),
        tile_set_name: tile_set_name.to_string(),
        tile_size,
        width,
        height,
        locks,
        random_cell_collapse_chance: 0.0,
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(WfcApp::new(config)))
        }),
    )
}
