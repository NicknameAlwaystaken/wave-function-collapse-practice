use rand::prelude::*;
use std::{cell::{Ref, RefCell}, rc::Rc, sync::Arc};

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
        Rc::clone(&house),
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
    ]);
    water.borrow_mut().allow(vec![
        Rc::clone(&error_cell),
        Rc::clone(&water),
        Rc::clone(&beach),
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
    ]);

    let all_cells: Vec<Rc<RefCell<Cell>>> = vec![
        Rc::clone(&tree),
        Rc::clone(&fire),
        Rc::clone(&house),
        Rc::clone(&grass),
        Rc::clone(&water),
        Rc::clone(&field),
        Rc::clone(&beach),
    ];

    let empty_cell = Rc::new(RefCell::new(Cell::new(' ')));

    let mut map = Vec::with_capacity(size_y);
    for _ in 0..size_y {
        let row: Vec<Rc<RefCell<Cell>>> = (0..size_x).map(|_| Rc::clone(&empty_cell)).collect();
        map.push(row);
    }

    let mut rng = rand::thread_rng();

    for x in 0..size_x {
        for y in 0..size_y {
            if !Rc::ptr_eq(&map[y][x], &empty_cell) {
                continue;
            }
            let mut allowed_cells = all_cells.clone();

            if x > 0 && !Rc::ptr_eq(&map[y][x - 1], &empty_cell) {
                allowed_cells.retain(|candidate| {
                    let neighbor = &map[y][x - 1];
                    let neighbor_borrow = neighbor.borrow();
                    let candidate_borrow = candidate.borrow();

                    neighbor_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, candidate))
                    &&
                    candidate_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, neighbor))
                });
            }

            if x + 1 < size_x && !Rc::ptr_eq(&map[y][x + 1], &empty_cell) {
                allowed_cells.retain(|candidate| {
                    let neighbor = &map[y][x + 1];
                    let neighbor_borrow = neighbor.borrow();
                    let candidate_borrow = candidate.borrow();

                    neighbor_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, candidate))
                    &&
                    candidate_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, neighbor))
                });
            }

            if y > 0 && !Rc::ptr_eq(&map[y - 1][x], &empty_cell) {
                allowed_cells.retain(|candidate| {
                    let neighbor = &map[y - 1][x];
                    let neighbor_borrow = neighbor.borrow();
                    let candidate_borrow = candidate.borrow();

                    neighbor_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, candidate))
                    &&
                    candidate_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, neighbor))
                });
            }

            if y + 1 < size_y && !Rc::ptr_eq(&map[y + 1][x], &empty_cell) {
                allowed_cells.retain(|candidate| {
                    let neighbor = &map[y + 1][x];
                    let neighbor_borrow = neighbor.borrow();
                    let candidate_borrow = candidate.borrow();

                    neighbor_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, candidate))
                    &&
                    candidate_borrow
                        .allowed_neighbours
                        .iter()
                        .any(|allowed| Rc::ptr_eq(allowed, neighbor))
                });
            }

            let new_cell = allowed_cells.choose(&mut rng);
            match new_cell {
                Some(cell) => {
                    map[y][x] = cell.clone();
                }
                None => {
                    // Debug if failed.
                    println!("No valid cell found for ({}, {})", x, y);
                    map[y][x] = Rc::clone(&error_cell);
                }
            }
        }
    }

    for row in &map {
        println!("{}", row.iter().map(|c| c.borrow().value).collect::<String>());
    }
}
