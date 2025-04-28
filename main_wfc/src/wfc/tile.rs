use core::panic;
use std::{collections::HashMap, path::Path, rc::Rc};

use image::RgbImage;
use walkdir::WalkDir;

use super::util::is_image_file;



impl TileSet {
    pub fn new() -> Self{
        Self {
            tiles: Vec::new(),
            name_lookup: HashMap::new(),
            ptr_lookup: HashMap::new(),
        }
    }

    fn add(&mut self, tile: Rc<Tile>) {
        self.tiles.push(Rc::clone(&tile));

        self.name_lookup.insert(tile.name.to_string(), Rc::clone(&tile));
        self.ptr_lookup.insert(Rc::as_ptr(&tile) as usize, Rc::clone(&tile));
    }

    pub fn add_list(&mut self, tile_list: Vec<Rc<Tile>>) {
        for tile in tile_list {
            self.add(tile);
        }
    }

    pub fn by_name(&self, name: &str) -> Option<Rc<Tile>> {
        self.name_lookup.get(name).cloned()
    }

    pub fn by_ptr(&self, ptr: usize) -> Option<Rc<Tile>> {
        self.ptr_lookup.get(&ptr).cloned()
    }
}

pub struct TileSet {
    pub tiles: Vec<Rc<Tile>>,
    pub name_lookup: HashMap<String, Rc<Tile>>,
    pub ptr_lookup: HashMap<usize, Rc<Tile>>,
}

#[derive(Clone, Debug)]
pub struct RotVariant {
    pub rot: u32,
    pub image: RgbImage,
    pub edges: [Vec<(u8, u8, u8)>; 4],
}

#[derive(Clone)]
pub struct Tile {
    pub name: String,
    pub variants: [RotVariant; 4],
}

pub fn get_tiles(
    root_folder: &str,
    tile_set_folder: &str,
) -> Vec<Rc<Tile>> {
    let source_folder = root_folder.to_string() + tile_set_folder;
    let image_paths: Vec<String> = WalkDir::new(&source_folder)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| is_image_file(entry.path()))
        .map(|entry| entry.path().display().to_string())
        .collect();

    let mut tiles: Vec<Rc<Tile>> = Vec::new();

    for image_path in image_paths {
        println!("{image_path}");
        let path = Path::new(&image_path);

        let image_name = match path.file_name()
            .and_then(|n| n.to_str()) {
            Some(path) => path,
            _ => {
                eprintln!("Couldn't get image name from {:?}", path);
                continue;
            }
        };

        let image_folder = match path.parent()
            .and_then(|p| p.to_str()) {
            Some(folder) => folder,
            _ => {
                eprintln!("Couldn't get folder path from {:?}", path);
                continue;
            }
        };

        println!("image_name: {}, path: {}", image_name, image_folder);

        tiles.push(Rc::new(image_to_tile(&image_name, &image_folder)));
    }

    if tiles.is_empty() {
        panic!("No tiles found! Please check if path is correct: {}", source_folder);
    }
    tiles
}

fn extract_edges(img: &RgbImage) -> [Vec<(u8, u8, u8)>; 4] {
    let (w, h) = (img.width(), img.height());

    let mut top = Vec::with_capacity(w as usize);
    let mut bottom = Vec::with_capacity(w as usize);
    let mut left = Vec::with_capacity(h as usize);
    let mut right = Vec::with_capacity(h as usize);

    for x in 0..w { top.push(img.get_pixel(x, 0).0.into()); }
    for x in 0..w { bottom.push(img.get_pixel(x, h - 1).0.into()); }
    for y in 0..h { left.push(img.get_pixel(0, y).0.into()); }
    for y in 0..h { right.push(img.get_pixel(w - 1, y).0.into()); }

    [right, bottom, left, top]
}

fn image_to_tile(
    file_name: &str,
    file_path: &str,
) -> Tile {
    let mut img = image::open(format!("{file_path}/{file_name}")).unwrap().to_rgb8();

    let mut variants = Vec::new();
        for rot in [0, 90, 180, 270] {
            if rot != 0 {
                img = image::imageops::rotate90(&img);   // rotate clockwise 90°
            }
            variants.push(RotVariant { rot,
                                       image: img.clone(),
                                       edges: extract_edges(&img) });
        }
        Tile { name: file_name.to_string(), variants: variants.try_into().unwrap() }
}
