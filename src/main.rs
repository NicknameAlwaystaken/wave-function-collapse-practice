use rand::prelude::*;
use core::panic;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
enum CellState {
    Collapsed(Rc<RefCell<Cell>>),
    Superposed(Vec<Rc<RefCell<Cell>>>),
}

pub struct Cell {
    pub value: char,
    pub allowed_neighbours: Vec<Rc<RefCell<Cell>>>,
}

impl Cell {
    fn new(value: char) -> Self {
        Self {
            value,
            allowed_neighbours: Vec::new(),
        }
    }

    fn allow(&mut self, neighbours: Vec<Rc<RefCell<Cell>>>) {
        self.allowed_neighbours = neighbours;
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

fn main() {
    let size_x = 50;
    let size_y = 25;

    let tree = Rc::new(RefCell::new(Cell::new('🌲')));
    let fire = Rc::new(RefCell::new(Cell::new('🔥')));
    let house = Rc::new(RefCell::new(Cell::new('🏡')));
    let grass = Rc::new(RefCell::new(Cell::new('🟩')));
    let water = Rc::new(RefCell::new(Cell::new('🌊')));
    let field = Rc::new(RefCell::new(Cell::new('🌾')));
    let beach = Rc::new(RefCell::new(Cell::new('🟨')));
    let cliff = Rc::new(RefCell::new(Cell::new('🪨')));
    let fish = Rc::new(RefCell::new(Cell::new('🐠')));
    let gem = Rc::new(RefCell::new(Cell::new('💎')));
    let error_cell = Rc::new(RefCell::new(Cell::new('❌')));

    tree.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&tree),
        Rc::clone(&fire),
        Rc::clone(&house),
        Rc::clone(&grass),
        Rc::clone(&water),
    ]);
    fire.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&fire),
        Rc::clone(&tree),
        Rc::clone(&house),
    ]);
    house.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&grass),
        Rc::clone(&tree),
    ]);
    grass.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&grass),
        Rc::clone(&tree),
        Rc::clone(&fire),
        Rc::clone(&house),
        Rc::clone(&field),
        Rc::clone(&beach),
        Rc::clone(&cliff),
    ]);
    water.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&water),
        Rc::clone(&beach),
        Rc::clone(&cliff),
        Rc::clone(&fish),
    ]);
    field.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&field),
        Rc::clone(&grass),
    ]);
    beach.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&beach),
        Rc::clone(&water),
        Rc::clone(&grass),
        Rc::clone(&cliff),
    ]);
    cliff.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&beach),
        Rc::clone(&water),
        Rc::clone(&grass),
        Rc::clone(&gem),
    ]);
    fish.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&water),
    ]);
    gem.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&cliff),
    ]);

    let all_cells: Vec<Rc<RefCell<Cell>>> = vec![
        Rc::clone(&tree),
        //Rc::clone(&fire),
        Rc::clone(&house),
        Rc::clone(&grass),
        Rc::clone(&water),
        Rc::clone(&field),
        Rc::clone(&beach),
        Rc::clone(&cliff),
        Rc::clone(&fish),
    ];

    let mut map: Vec<Vec<CellState>> = vec![vec![CellState::Superposed(all_cells.clone()); size_x]; size_y];

    let mut rng = rand::thread_rng();

    loop {
        let mut lowest_entropy = usize::MAX;

        let mut candidates = vec![];

        for x in 0..size_x {
            for y in 0..size_y {
                match &map[y][x] {
                    CellState::Superposed(options) => {
                        let entropy = options.len();
                        if entropy < lowest_entropy {
                            lowest_entropy = entropy;
                            candidates.clear();
                            candidates.push((x, y));
                        } else if entropy == lowest_entropy {
                            candidates.push((x, y));
                        }
                    }
                    CellState::Collapsed(_) => {}
                }
            }
        }

        let &(x, y) = candidates.choose(&mut rng).unwrap();

        let options = match &map[y][x] {
            CellState::Superposed(opts) => opts,
            _ => panic!("Expected Superposed cell"),
        };

        let chosen_tile = options.choose(&mut rng).unwrap().clone();
        map[y][x] = CellState::Collapsed(chosen_tile);

        if map.iter().all(|row| row.iter().all(|cell| matches!(cell, CellState::Collapsed(_)))) {
            break;
        }

        let mut to_propagate = vec![(x, y)];

        while let Some((cx, cy)) = to_propagate.pop() {
            let collapsed = match &map[cy][cx] {
                CellState::Collapsed(c) => Rc::clone(c),
                _ => continue,
            };

            for (nx, ny) in neighbours(cx, cy, size_x, size_y) {
                if let CellState::Superposed(options) = &mut map[ny][nx] {
                    let before = options.len();
                    options.retain(|opt| {
                        let opt_borrow = opt.borrow();
                        opt_borrow.allowed_neighbours.iter().any(|n| Rc::ptr_eq(n, &collapsed))
                            && collapsed.borrow().allowed_neighbours.iter().any(|n| Rc::ptr_eq(n, opt))
                    });

                    if options.is_empty() {
                        panic!("No valid options left during propagation at ({}, {})", nx, ny);
                    }

                    if options.len() < before {
                        to_propagate.push((nx, ny));
                    }
                }
            }
        }
    }

    for row in &map {
        println!("{}", row.iter()
            .map(|c| match c {
                CellState::Collapsed(tile) => tile.borrow().value,
                CellState::Superposed(_) => '·',
            })
            .collect::<String>());
    }
}

fn neighbours(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(4);

    if x > 0 {
        result.push((x - 1, y));
    }
    if x + 1 < width {
        result.push((x + 1, y));
    }
    if y > 0 {
        result.push((x, y - 1));
    }
    if y + 1 < height {
        result.push((x, y + 1));
    }

    result
}
