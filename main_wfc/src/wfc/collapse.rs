use std::{collections::HashSet, rc::Rc};

use rand::{distr::{weighted::WeightedIndex, Distribution}, seq::SliceRandom, Rng};

use crate::wfc::{rules::find_compatible_neighbours, util::{neighbour_coords, opposite}, wave::WaveState};

use super::engine::WfcEngine;


pub fn collapse(engine: &mut WfcEngine) -> Result<(), ()> {

    let tile_set = &engine.tile_set;
    let width = engine.width;
    let height = engine.height;
    let _locks = &engine.locks;
    let weight_map = &engine.weight_map;
    let noise_grid = &engine.noise_grid;
    let rule_map = &engine.rule_map;
    let rng = &mut engine.rng;
    let history = &mut engine.history;

    let mut lowest_entropy = usize::MAX;

    let mut candidates: Vec<(usize, usize)> = vec![];

    for x in 0..width {
        for y in 0..height {
            let cell = &engine.wave.grid[y][x].borrow();
            if cell.collapsed {
                continue;
            }

            let entropy = cell.options.len();
            if entropy < lowest_entropy {
                lowest_entropy = entropy;
                candidates.clear();
                candidates.push((x, y));
            } else if entropy == lowest_entropy {
                candidates.push((x, y));
            }
        }
    }

    let random_f32 = &rng.random::<f32>();

    let (x, y) = if random_f32 < &engine.random_cell_collapse_chance {
        let mut noisy_candidates = vec![];
        for x in 0..width {
            for y in 0..height {
                let cell = &engine.wave.grid[y][x].borrow();
                if !cell.collapsed {
                    noisy_candidates.push((x, y));
                }
            }
        }

        noisy_candidates
            .into_iter()
            .max_by(|&(ax, ay), &(bx, by)| {
                noise_grid[ay][ax]
                    .partial_cmp(&noise_grid[by][bx])
                    .unwrap()
            })
            .unwrap()
    } else {
        *candidates
            .iter()
            .max_by(|&&(ax, ay), &&(bx, by)| {
                noise_grid[ay][ax]
                    .partial_cmp(&noise_grid[by][bx])
                    .unwrap()
            })
            .unwrap()
    };

    let chosen_tile = {
        let options = {
            let cell = engine.wave.grid[y][x].borrow();
            cell.options.clone()
        };

        let weights: Vec<f32> = options.iter().map(|(tile, rot, _)| {
            let tile_key = Rc::as_ptr(tile) as usize;
            let base_weight = *weight_map.get(&(tile_key, *rot)).unwrap_or(&100.0);

            let used = engine.seen.get(&tile_key).unwrap_or(&0);
            let eff_weight = base_weight / (1.0 + *used as f32 * 0.01);
            eff_weight.max(0.0001)
        }).collect();

        let idx = if engine.first_tile {
            engine.first_tile = false;
            rng.random_range(0..options.len())
        } else {
            let dist = WeightedIndex::new(&weights).unwrap();
            dist.sample(rng.as_mut())
        };

        let (t, r, _) = &options[idx];
        let chosen_tile = Rc::clone(t);
        let rotation = *r;

        let mut updated_options = options.clone();
        updated_options.remove(idx);
        updated_options.shuffle(rng.as_mut());

        /*
        println!("→ Collapsing ({}, {}) {}@{}, options were:", x, y, chosen_tile.name, rotation);
        for (tile, rot, _) in &wave.grid[y][x].borrow().options {
            println!("  - {} @{}", tile.name, rot);
        }
        */

        {
            let mut cell = engine.wave.grid[y][x].borrow_mut();
            cell.options = vec![(Rc::clone(&chosen_tile), rotation, 1.0)];
            cell.collapsed = true;
        }

        history.push(WaveState {
            wave: engine.wave.deep_clone(),
            seen: engine.seen.clone(),
            x,
            y,
            remaining_options: updated_options,
        });

        chosen_tile
    };

    let k = Rc::as_ptr(&chosen_tile) as usize;
    *engine.seen.entry(k).or_insert(0) += 1;

    let mut to_propagate = vec![(x, y)];

    while let Some((cx, cy)) = to_propagate.pop() {
        let (ref source_tile, source_rot, _) = {
            let source_cell = &mut engine.wave.grid[cy][cx].borrow();

            if !source_cell.collapsed {
                continue;
            }

            let idx = rng.random_range(0..source_cell.options.len());
            source_cell.options[idx].clone()
        };

        for (nx, ny, neigh_dir) in neighbour_coords(cx, cy, width, height) {
            let valid_neighbours = {
                let neighbour_cell = &mut engine.wave.grid[ny][nx].borrow_mut();

                if neighbour_cell.collapsed {
                    continue;
                }

                // gather ALL possible neighbour tiles based on source_tile,
                // later the possible tiles are filtered by their own neighbours
                find_compatible_neighbours(
                    nx,
                    ny,
                    neigh_dir,
                    Rc::clone(source_tile),
                    source_rot,
                    &tile_set,
                    &rule_map,
                    &engine.wave,
                )
            };

            /*
            let valid_set: HashSet<(usize, u32)> = valid_neighbours
                .iter()
                .map(|(tile, rot)| (Rc::as_ptr(tile) as usize, *rot))
                .collect();
            */

            let valid_set: HashSet<(usize, u32)> = valid_neighbours
                .into_iter()
                .filter(|&(ref tile, rot)| {
                    let tile_ptr = Rc::as_ptr(tile) as usize;

                    // For each direction (Right, Bottom, etc)
                    for (adj_x, adj_y, dir_to_candidate) in neighbour_coords(nx, ny, width, height) {
                        let adj_cell = engine.wave.grid[adj_y][adj_x].borrow();

                        if !adj_cell.collapsed {
                            continue;
                        }

                        let (adj_tile, adj_rot, _) = adj_cell.options[0].clone();

                        // Lookup expected neighbors from the collapsed neighbor
                        let neighbor_options = find_compatible_neighbours(
                            nx,
                            ny,
                            opposite(dir_to_candidate),
                            Rc::clone(&adj_tile),
                            adj_rot,
                            tile_set,
                            rule_map,
                            &engine.wave,
                        );

                        let allowed_set: HashSet<(usize, u32)> = neighbor_options
                            .into_iter()
                            .map(|(t, r)| (Rc::as_ptr(&t) as usize, r))
                            .collect();


                        /*
                        println!("   Allowed by {}@{}:", adj_tile.name, adj_rot);
                        for (t, r) in &allowed_set {
                            println!("     - {} @{}", tile_set.by_ptr(*t).unwrap().name, r);
                        }
                        */

                        if !allowed_set.contains(&(tile_ptr, rot)) {
                            //println!("🚨 Invalid neighbor match: {}@{} not allowed by {}@{}", tile.name, rot, adj_tile.name, adj_rot);
                            //thread::sleep(Duration::from_millis(1000));
                            return false;
                        }

                        //thread::sleep(Duration::from_millis(1000));
                    }

                    true // all adjacent constraints passed
                })
                .map(|(tile, rot)| (Rc::as_ptr(&tile) as usize, rot))
                .collect();


            let filtered_options: Vec<_> = {
                let neighbour_cell = &mut engine.wave.grid[ny][nx].borrow_mut();

                neighbour_cell
                    .options
                    .iter()
                    .filter(|(tile, rot, _)| {
                        let ptr = Rc::as_ptr(tile) as usize;
                        valid_set.contains(&(ptr, *rot))
                        //&& is_still_globally_valid((Rc::clone(tile), *rot), nx, ny, &wave, tile_set, rule_map)
                    })
                    .cloned()
                    .collect()
            };

            if filtered_options.is_empty() {

                while let Some(mut state) = history.pop() {
                    if let Some((next_tile, next_rot, _)) = state.remaining_options.pop() {
                        /*
                        println!("🔁 Backtracking: trying next tile for ({}, {})", state.x, state.y);
                        println!("   ↪️ Trying tile: {} @{}", next_tile.name, next_rot);
                        println!("   📦 Remaining options at this cell before retry:");

                        for (tile, rot, _) in &state.remaining_options {
                            println!("     - {} @{}", tile.name, rot);
                        }
                        */

                        engine.wave = state.wave;
                        engine.seen = state.seen;

                        {
                            let mut cell = engine.wave.grid[state.y][state.x].borrow_mut();
                            /*
                            println!("   💠 Previous cell state being replaced at ({}, {}):", state.x, state.y);
                            for (t, r, _) in &cell.options {
                                println!("     - {} @{}", t.name, r);
                            }
                            */

                            cell.options = vec![(Rc::clone(&next_tile), next_rot, 1.0)];
                            cell.collapsed = true;
                        }

                        let k = Rc::as_ptr(&next_tile) as usize;
                        *engine.seen.entry(k).or_insert(0) += 1;

                        history.push(WaveState {
                            wave: engine.wave.deep_clone(),
                            seen: engine.seen.clone(),
                            x: state.x,
                            y: state.y,
                            remaining_options: state.remaining_options,
                        });

                        to_propagate.clear();
                        to_propagate.push((state.x, state.y));
                        break;
                    }
                }

                if history.is_empty() {
                    println!("Out of tiles to try.");
                    //thread::sleep(Duration::from_millis(2000));
                    return Err(());
                }

                //thread::sleep(Duration::from_millis(1000));
                continue;
            }

            {
                let neighbour_cell = &mut engine.wave.grid[ny][nx].borrow_mut();

                let options_changed = neighbour_cell.options.len() != filtered_options.len()
                    || !neighbour_cell.options.iter().all(|(tile1, rot1, _)| {
                    filtered_options.iter().any(|(tile2, rot2, _)| {
                        Rc::ptr_eq(tile1, tile2) && rot1 == rot2
                    })
                });

                if options_changed {
                    neighbour_cell.options = filtered_options;
                    to_propagate.push((nx, ny));
                }
            }
        }
    }

    let collapsed = engine.wave.grid
        .iter()
        .flatten()
        .filter(|c| c.borrow().collapsed)
        .count();

    let total = width * height;

    engine.progress = Some((collapsed, total));

    if engine.wave.grid.iter().all(|row| row.iter().all(|cell| cell.borrow().collapsed)) {
        println!("");
        println!("All cells successfully collapsed!");
        engine.solved = true;
        return Ok(());
    }

    Ok(())
}
