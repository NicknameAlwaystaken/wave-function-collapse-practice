use std::{collections::HashMap, path::Path};

use eframe::egui::{Color32, ColorImage};
use rand::RngCore;

use super::{render::{blit_tile_onto_colorimage, blit_tile_onto_rgbimage}, rules::load_json_rules, tile::{get_tiles, TileSet}, util::{make_noise, Direction}, wave::{Locked, Wave, WaveState}, weight::load_weights};

#[derive(Clone)]
pub struct WfcEngineConfig {
    pub tile_path: String,
    pub tile_set_name: String,
    pub tile_size: u32,
    pub width: usize,
    pub height: usize,
    pub locks: Vec<Locked>,
    pub random_cell_collapse_chance: f32,
}

pub struct WfcEngine {
    pub wave: Wave,
    pub tile_set: TileSet,
    pub history: Vec<WaveState>,
    pub rule_map: HashMap<(usize, Direction), Vec<(usize, u32)>>,
    pub weight_map: HashMap<(usize, u32), f32>,
    pub noise_grid: Vec<Vec<f64>>,
    pub seen: HashMap<usize, u32>,
    pub locks: Vec<Locked>,
    pub tile_size: u32,
    pub width: usize,
    pub height: usize,
    pub first_tile: bool,
    pub random_cell_collapse_chance: f32,
    pub rng: Box<dyn RngCore>,
    pub progress: Option<(usize, usize)>,
    pub solved: bool,
}

impl WfcEngine {
    pub fn new(config: WfcEngineConfig) -> Self {
        let full_path = Path::new(&config.tile_path).join(&config.tile_set_name);
        let rule_path = full_path.join("rules.json");

        let mut tile_set = TileSet::new();
        tile_set.add_list(get_tiles(&config.tile_path, &config.tile_set_name));

        let weight_map = load_weights("weights.txt", &tile_set);
        let rule_map = load_json_rules(&rule_path, &tile_set);
        let noise_grid = make_noise(config.width, config.height, 1.0 / config.width.max(config.height) as f64);
        let wave = Wave::new(&tile_set, config.width, config.height, &weight_map);

        Self {
            wave,
            tile_set,
            history: Vec::new(),
            rule_map,
            weight_map,
            noise_grid,
            seen: HashMap::new(),
            locks: config.locks.clone(),
            tile_size: config.tile_size,
            width: config.width,
            height: config.height,
            random_cell_collapse_chance: config.random_cell_collapse_chance,
            first_tile: true,
            rng: Box::new(rand::rng()),
            progress: None,
            solved: false,
        }
    }

    pub fn draw(&mut self) -> ColorImage {
        let mut color_image = ColorImage::new(
            [
                (self.width as u32 * self.tile_size) as usize,
                (self.height as u32 * self.tile_size) as usize,
            ],
            Color32::BLACK,
        );

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.wave.grid[y][x].borrow();
                if let Some((tile, rotation)) = cell.collapsed_option() {
                    let var = &tile.variants[(rotation % 360 / 90) as usize];
                    blit_tile_onto_colorimage(
                        &mut color_image,
                        &var.image,
                        x as u32 * var.image.width(),
                        y as u32 * var.image.height(),
                    );
                }
            }
        }

        color_image
    }

}
