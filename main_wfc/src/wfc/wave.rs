use std::{cell::RefCell, collections::HashMap, rc::Rc};
use rand::{rng, seq::SliceRandom};

use super::tile::{Tile, TileSet};


#[derive(Clone)]
pub struct Locked {
    pub x: usize,
    pub y: usize,
    pub tile: Rc<Tile>,
    pub rot: u32,
}

pub struct WaveState {
    pub wave: Wave,
    pub seen: HashMap<usize, u32>,
    pub x: usize,
    pub y: usize,
    pub remaining_options: Vec<(Rc<Tile>, u32, f32)>,
}

#[derive(Clone)]
pub struct WaveCell {
    pub options: Vec<(Rc<Tile>, u32, f32)>, // tile, rotation, weight
    pub collapsed: bool,
}

#[derive(Clone)]
pub struct Wave {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<RefCell<WaveCell>>>,
}

impl Wave {
    pub fn new(
        tile_set: &TileSet,
        width: usize,
        height: usize,
        weight_map: &HashMap<(usize, u32), f32>,
    ) -> Self {
        let mut rng = rng();
        let mut grid = vec![];
        for _ in 0..height {
            let mut row = vec![];
            for _ in 0..width {
                let mut options: Vec<(Rc<Tile>, u32, f32)> = vec![];
                for tile in &tile_set.tiles {
                    for rot in [0, 90, 180, 270] {
                        let w = weight_map
                            .get(&(Rc::as_ptr(tile) as usize, rot))
                            .cloned()
                            .unwrap_or(1.0);
                        options.push((Rc::clone(tile), rot, w));
                    }
                }

                options.shuffle(&mut rng);

                row.push(RefCell::new(WaveCell {
                    options,
                    collapsed: false,
                }));
            }
            grid.push(row);
        }

        Self {
            width,
            height,
            grid,
        }
    }

    pub fn deep_clone(&self) -> Self {
        let grid = self.grid.iter()
            .map(|row| {
                row.iter()
                    .map(|cell| {
                        let cell_ref = cell.borrow();
                        RefCell::new(cell_ref.clone()) // deep clone the WaveCell
                    })
                    .collect()
            })
            .collect();

        Self {
            width: self.width,
            height: self.height,
            grid,
        }
    }
}

impl WaveCell {
    pub fn collapsed_option(&self) -> Option<(&Rc<Tile>, u32)> {
        if self.collapsed {
            if self.options.len() != 1 {
                panic!("Cell marked as collapsed but has {} options!", self.options.len());
            }

            let (t, r, _) = self.options.first().unwrap();
            Some((t, *r))
        } else {
            None
        }
    }
}

pub struct Cell {
    pub value: char,
    pub allowed_neighbours: Vec<Rc<RefCell<Cell>>>,
}
