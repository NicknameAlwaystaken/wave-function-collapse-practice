use std::{collections::HashMap, path::Path, rc::Rc};

use super::tile::TileSet;


pub fn load_weights(file: &str, tile_set: &TileSet) -> HashMap<(usize, u32), f32> {
    let mut map = HashMap::<(usize, u32), f32>::new();

    if let Ok(text) = std::fs::read_to_string(file) {
        println!("Weights file found! Reading...");
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let name = match parts.next() {
                Some(s) => s,
                None => continue,
            };
            let weight = parts.next().and_then(|w| w.parse::<f32>().ok()).unwrap_or(100.0);

            for tile in &tile_set.tiles {
                if Path::new(&tile.name)
                    .file_name()
                    .and_then(|s| s.to_str()) == Some(name)
                {
                    for rot in [0, 90, 180, 270] {
                        map.insert((Rc::as_ptr(tile) as usize, rot), weight);
                        println!("Weight found for {}: {}", name, weight);
                    }

                }
            }
        }
    }
    map
}
