use std::{fs, path::Path};

use noise::{NoiseFn, Perlin};

use super::tile::Tile;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Right,
    Bottom,
    Left,
    Top,
}

pub fn get_available_tile_sets(tile_path: &str) -> Vec<String> {
    let mut sets = Vec::new();

    if let Ok(entries) = fs::read_dir(tile_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    sets.push(name.to_string());
                }
            }
        }
    }

    sets
}

pub fn dir_index(d: Direction) -> i32 {
    match d {
        Direction::Top => 0,
        Direction::Right => 1,
        Direction::Bottom => 2,
        Direction::Left => 3,
    }
}

pub fn make_noise(width: usize, height: usize, scale: f64) -> Vec<Vec<f64>> {
    let perlin = Perlin::new(42);
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    let v = perlin.get([x as f64 * scale, y as f64 * scale]);
                    0.5 * (v + 1.0)
                })
                .collect()
        })
        .collect()
}

pub fn opposite(dir: Direction) -> Direction {
    match dir {
        Direction::Right => Direction::Left,
        Direction::Bottom => Direction::Top,
        Direction::Left => Direction::Right,
        Direction::Top => Direction::Bottom,
    }
}

pub fn str_to_dir(string: &str) -> Option<Direction> {
    match string {
        "right" | "Right" => Some(Direction::Right),
        "bottom" | "Bottom" => Some(Direction::Bottom),
        "left" | "Left" => Some(Direction::Left),
        "top" | "Top" => Some(Direction::Top),
        _ => None,
    }
}

pub fn clockwise_next_direction(dir: Direction) -> Direction {
    match dir {
        Direction::Right => Direction::Bottom,
        Direction::Bottom => Direction::Left,
        Direction::Left => Direction::Top,
        Direction::Top => Direction::Right,
    }
}

pub fn anticlockwise_next_direction(dir: Direction) -> Direction {
    match dir {
        Direction::Right => Direction::Top,
        Direction::Bottom => Direction::Right,
        Direction::Left => Direction::Bottom,
        Direction::Top => Direction::Left,
    }
}

pub fn dir_to_idx(d: Direction) -> usize {
    match d {
        Direction::Right => 0,
        Direction::Bottom => 1,
        Direction::Left => 2,
        Direction::Top => 3,
    }
}

pub fn rotate_direction_from_ccw(direction: Direction, rotation: u32) -> Direction {
    let mut steps = (rotation / 90) % 4;
    let mut dir = direction;
    while steps > 0 {
        dir = anticlockwise_next_direction(dir);
        steps -= 1;
    }
    dir
}

pub fn get_rotated_edge(tile: &Tile, rot: u32, direction: Direction) -> Vec<(u8,u8,u8)> {
    let var = &tile.variants[(rot % 360 / 90) as usize];
    var.edges[dir_to_idx(direction)].clone()
}

pub fn neighbour_coords(
    x: usize, y: usize, width: usize, height: usize
) -> Vec<(usize, usize, Direction)> {
    let mut result = Vec::with_capacity(4);

    if x > 0 {
        result.push((x - 1, y, Direction::Left));
    }
    if x + 1 < width {
        result.push((x + 1, y, Direction::Right));
    }
    if y > 0 {
        result.push((x, y - 1, Direction::Top));
    }
    if y + 1 < height {
        result.push((x, y + 1, Direction::Bottom));
    }

    result
}

pub fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "bmp" | "gif"),
        None => false,
    }
}
