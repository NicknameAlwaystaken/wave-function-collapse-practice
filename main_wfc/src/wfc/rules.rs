use std::{collections::{HashMap, HashSet}, fs, path::Path, rc::Rc};
use serde::Deserialize;

use crate::wfc::util::str_to_dir;

use super::{tile::{Tile, TileSet}, util::{dir_to_idx, get_rotated_edge, neighbour_coords, opposite, rotate_direction_from_ccw, Direction}, wave::{Wave, WaveCell}};

#[derive(Debug, Deserialize, Clone, PartialEq)]
struct JsonRule {
    name: String,
    rotation: u32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RuleEntry {
    Include { include: String, exclude: Vec<JsonExclude> },
    Allow { allow: Vec<JsonRule> },
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
enum JsonExclude {
    Name(String),
    NameAndRotation { name: String, rotation: u32 },
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
enum JsonAllow {
    Name(String),
    NameAndRotation { name: String, rotation: u32 },
}

#[derive(Debug, Deserialize)]
struct JsonTileRules {
    connections: HashMap<String, HashMap<String, Vec<JsonRule>>>,
    tiles: HashMap<String, HashMap<String, RuleEntry>>,
}

fn is_excluded_by_name_or_rotation(
    rule: &JsonRule,
    exclude: &[JsonExclude],
) -> bool {
    exclude.iter().any(|e| {
        match e {
            JsonExclude::Name(name) => &rule.name == name,
            JsonExclude::NameAndRotation { name, rotation } => {
                &rule.name == name && rule.rotation == *rotation
            },
        }
    })
}

fn is_allowed_by_name_or_rotation(
    rule: &JsonRule,
    allow: &[JsonAllow],
) -> bool {
    allow.iter().any(|e| {
        match e {
            JsonAllow::Name(name) => &rule.name == name,
            JsonAllow::NameAndRotation { name, rotation } => {
                &rule.name == name && rule.rotation == *rotation
            },
        }
    })
}

pub fn load_json_rules(
    rules_path: &Path,
    tile_set: &TileSet,
) -> HashMap<(usize, Direction), Vec<(usize, u32)>> {
    let file_content = fs::read_to_string(rules_path)
        .expect("Failed to read rules.json");

    let parsed: JsonTileRules = serde_json::from_str(&file_content)
        .expect("Failed to parse rules.json");

    let mut rule_map = HashMap::new();

    for (tile_name, dir_map) in parsed.tiles {
        let tile = match tile_set.by_name(&tile_name) {
            Some(t) => t,
            None => continue,
        };

        for (dir_str, entry) in dir_map {
            let dir = match str_to_dir(&dir_str) {
                Some(d) => d,
                None => continue,
            };

            let variants = match entry {
                RuleEntry::Allow { allow } => {
                    allow
                }
                RuleEntry::Include { include, exclude } => {
                    parsed
                        .connections
                        .get(&include)
                        .and_then(|map| map.get(&dir_str))
                        .map(|entries| {
                            entries
                                .iter()
                                .filter(|e| !is_excluded_by_name_or_rotation(e, &exclude))
                                .cloned()
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                }
            };

            let ptr = Rc::as_ptr(&tile) as usize;
            let entries = variants
                .into_iter()
                .filter_map(|e| tile_set.by_name(&e.name).map(|t| (Rc::as_ptr(&t) as usize, e.rotation)))
                .collect();

            rule_map.insert((ptr, dir), entries);
        }
    }

    rule_map
}


pub fn load_tile_rules(file: &str, tile_set: &TileSet) -> HashMap<(usize, Direction), Vec<(usize, u32)>> {
    let mut tile_rule_map: HashMap<(usize, Direction), Vec<(usize, u32)>> = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(file) {
        println!("Tile rules file found! Reading...");
        for line in text.lines() {
            let parts: Vec<_> = line.split_whitespace().collect();
            match parts.as_slice() {
                [tile_name, direction, fitting_tile_name, neighbour_rotation] => {
                    let tile = match tile_set.name_lookup.get(*tile_name) {
                        Some(tile) => tile,
                        _ => {
                            eprintln!("Tile by name {} was not found.", tile_name);
                            continue;
                        }
                    };

                    let fitting_tile = match tile_set.name_lookup.get(*fitting_tile_name) {
                        Some(fitting_tile) => fitting_tile,
                        _ => {
                            eprintln!("Fitting tile by name {} was not found.", fitting_tile_name);
                            continue;
                        }
                    };

                    let direction: Direction = match str_to_dir(direction) {
                        Some(direction) => direction,
                        _ => {
                            eprintln!("Invalid direction: {}", direction);
                            continue;
                        }
                    };

                    let neighbour_rotation: u32 = match neighbour_rotation.parse::<u32>() {
                        Ok(r) if [0, 90, 180, 270].contains(&r) => r,
                        _ => {
                            eprintln!("Invalid rotation: {}", neighbour_rotation);
                            continue;
                        }
                    };
                    println!("Found tile {}", tile.name);

                    let key = (Rc::as_ptr(tile) as usize, direction);
                    let value = (Rc::as_ptr(fitting_tile) as usize, neighbour_rotation);

                    tile_rule_map.entry(key).or_default().push(value);
                }
                _ => {
                    eprintln!("Skipping malformed rule line: {line}");
                }
            }
            println!("{line}");
        }
    }

    tile_rule_map
}

pub fn find_compatible_neighbours(
    x: usize,
    y: usize,
    source_dir: Direction,
    source_tile: Rc<Tile>,
    source_rot: u32,
    tile_set: &TileSet,
    rule_map: &HashMap<(usize, Direction), Vec<(usize, u32)>>,
    wave: &Wave,
) -> Vec<(Rc<Tile>, u32)> {
    let tile_ptr = Rc::as_ptr(&source_tile) as usize;
    let rotated_dir = rotate_direction_from_ccw(source_dir, source_rot);
    let rule_key = (tile_ptr, rotated_dir);

    if let Some(rules) = rule_map.get(&rule_key) {
        return rules
            .iter()
            .filter_map(|(neighbour_ptr, rotation)| {
                tile_set.by_ptr(*neighbour_ptr).map(|tile_rc| {
                    let final_rotation = (source_rot + rotation) % 360;

                    (tile_rc, final_rotation)
                })
            })
            .collect();
    } else {
        println!(
            "⚠️ No rules found for '{}', dir {:?} (rotated: {:?})",
            source_tile.name, source_dir, rotated_dir
        );
        vec![]
    }

    /*


    let source_edge = get_rotated_edge(&source_tile, source_rot, source_dir);

    /*
    tile_set.tiles
        .iter()
        .flat_map(|tile| [0, 90, 180, 270].into_iter().map(move |rot| (tile, rot)))
        .filter(|(tile, rot)| {
            let edge = get_rotated_edge(tile, *rot, opposite(direction));
            edge == source_edge
        })
        .map(|(tile, rot)| (Rc::clone(tile), rot))
        .collect()
    */

    let mut candidates = vec![];

    for tile in &tile_set.tiles {
        for rot in [0, 90, 180, 270] {
            let target_edge = get_rotated_edge(tile, rot, opposite(source_dir));

            if source_edge == target_edge {
                candidates.push((tile.clone(), rot));
            }
        }
    }

    candidates
    */
}

fn retain_compatible(
    source_edges: &[Vec<(u8, u8, u8)>; 4],
    dir_from_src: Direction,
    neigh: &mut WaveCell,
) -> bool {
    let before = neigh.options.len();

    neigh.options.retain(|(tile, rot, _)| {
        let target_edge = get_rotated_edge(tile, *rot, opposite(dir_from_src));
        source_edges[dir_to_idx(dir_from_src)] == target_edge
    });

    before != neigh.options.len()
}

fn is_still_globally_valid(
    candidate: (Rc<Tile>, u32),
    x: usize,
    y: usize,
    wave: &Wave,
    tile_set: &TileSet,
    rule_map: &HashMap<(usize, Direction), Vec<(usize, u32)>>,
) -> bool {
    let (ref tile, rotation) = candidate;
    let tile_ptr = Rc::as_ptr(tile) as usize;

    for (nx, ny, dir_to_candidate) in neighbour_coords(x, y, wave.width, wave.height) {
        let neighbor = wave.grid[ny][nx].borrow();

        if neighbor.collapsed {
            let (ref neighbor_tile, neighbor_rot, _) = neighbor.options[0];

            // What tiles does the neighbor expect at its dir_from_candidate?
            let allowed_tiles = find_compatible_neighbours(
                nx,
                ny,
                opposite(dir_to_candidate),
                Rc::clone(neighbor_tile),
                neighbor_rot,
                tile_set,
                rule_map,
                wave,
            );

            let allowed_set: HashSet<(usize, u32)> = allowed_tiles
                .into_iter()
                .map(|(t, r)| (Rc::as_ptr(&t) as usize, r))
                .collect();

            if !allowed_set.contains(&(tile_ptr, rotation)) {
                return false;
            }
        }
    }

    true
}
