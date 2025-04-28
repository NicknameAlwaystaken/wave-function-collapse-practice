use std::{borrow::{Borrow, BorrowMut}, cell::{RefCell, RefMut}, collections::HashMap, fs, ops::Sub, rc::Rc};

use eframe::{egui::{self, vec2, TextureHandle}, epaint::Vertex};
use serde::Serialize;
use serde_json::{json, Value};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

#[derive(Clone, PartialEq)]
struct Tile {
    name: String,
    texture: egui::TextureHandle,
    connections: PatternConnections,
}

#[derive(Clone, PartialEq)]
struct Cell {
    kind: CellKind,
}

#[derive(Clone, PartialEq)]
struct CommonPattern {
    name: String,
    connections: PatternConnections,
}

struct TileRulesEditorApp {
    tile_path: String,
    tile_set_name: String,
    tiles: HashMap<String, Rc<RefCell<Tile>>>,
    symbols_path: String,
    symbols: HashMap<String, egui::TextureHandle>,
    selected: Option<Selection>,
    selection_state: SelectionState,
    selected_plus: Option<Selection>,
    tile_zoom: f32,
    common_patterns: HashMap<String, Rc<RefCell<CommonPattern>>>,
}

#[derive(Clone, PartialEq)]
struct PatternConnections {
    top: Vec<PatternEntry>,
    right: Vec<PatternEntry>,
    left: Vec<PatternEntry>,
    bottom: Vec<PatternEntry>,
}

/// What one neighbour looks like in the output.
#[derive(Serialize)]
struct Entry<'a> {
    name: &'a str,
    rotation: u16,
}

#[derive(EnumIter, Clone, PartialEq, Debug, Copy)]
enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

#[derive(Clone, PartialEq)]
enum PatternEntry {
    Tile {
        tile: Rc<RefCell<Tile>>,
        rotation: Rotation,
    },
    CommonPattern(Rc<RefCell<CommonPattern>>),
}

#[derive(Clone, PartialEq)]
enum Selection {
    Tile(Rc<RefCell<Tile>>),
    CommonPattern(Rc<RefCell<CommonPattern>>),
    Insert(Direction),
}

#[derive(Clone, PartialEq)]
enum CellKind {
    Tile {
        texture: TextureHandle,
        rotation: Rotation,
    },
    Plus,
    Empty,
}

enum SelectionState {
    Normal,
    Inserting,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Top,
    Right,
    Bottom,
    Left,
}

impl Direction {
   pub const fn opposite(self) -> Direction {
        match self {
            Direction::Top => Direction::Bottom,
            Direction::Bottom => Direction::Top,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    pub fn rotated_cw(self) -> Self {
        match self {
            Direction::Top => Direction::Right,
            Direction::Right => Direction::Bottom,
            Direction::Bottom => Direction::Left,
            Direction::Left => Direction::Top,
        }
    }

    pub fn rotated_ccw(self) -> Self {
        match self {
            Direction::Top => Direction::Left,
            Direction::Right => Direction::Top,
            Direction::Bottom => Direction::Right,
            Direction::Left => Direction::Bottom,
        }
    }

    pub fn rotated_cw_by(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::R0 => self,
            Rotation::R90 => self.rotated_cw(),
            Rotation::R180 => self.rotated_cw().rotated_cw(),
            Rotation::R270 => self.rotated_cw().rotated_cw().rotated_cw(),
        }
    }

    pub fn rotated_ccw_by(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::R0 => self,
            Rotation::R90 => self.rotated_ccw(),
            Rotation::R180 => self.rotated_ccw().rotated_ccw(),
            Rotation::R270 => self.rotated_ccw().rotated_ccw().rotated_ccw(),
        }
    }
}

impl Rotation {
    pub fn rotated_cw(self) -> Self {
        match self {
            Rotation::R0 => Rotation::R90,
            Rotation::R90 => Rotation::R180,
            Rotation::R180 => Rotation::R270,
            Rotation::R270 => Rotation::R0,
        }
    }

    pub fn rotated_ccw(self) -> Self {
        match self {
            Rotation::R0 => Rotation::R270,
            Rotation::R90 => Rotation::R0,
            Rotation::R180 => Rotation::R90,
            Rotation::R270 => Rotation::R180,
        }
    }

    pub fn rotated_cw_by(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::R0 => self,
            Rotation::R90 => self.rotated_cw(),
            Rotation::R180 => self.rotated_cw().rotated_cw(),
            Rotation::R270 => self.rotated_cw().rotated_cw().rotated_cw(),
        }
    }

    pub fn rotated_ccw_by(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::R0 => self,
            Rotation::R90 => self.rotated_ccw(),
            Rotation::R180 => self.rotated_ccw().rotated_ccw(),
            Rotation::R270 => self.rotated_ccw().rotated_ccw().rotated_ccw(),
        }
    }

    fn degrees(self) -> u16 {
        match self {
            Rotation::R0 => 0,
            Rotation::R90 => 90,
            Rotation::R180 => 180,
            Rotation::R270 => 270,
        }
    }

    const fn opposite(self) -> Self {
        match self {
            Rotation::R0 => Rotation::R180,
            Rotation::R180 => Rotation::R0,
            Rotation::R90 => Rotation::R270,
            Rotation::R270 => Rotation::R90,
        }
    }
}

impl Sub for Rotation {
    type Output = Rotation;

    fn sub(self, rhs: Self) -> Self::Output {
        let a = self.degrees() as i32;
        let b = rhs.degrees() as i32;
        let diff = (a - b).rem_euclid(360);
        match diff {
           0 => Rotation::R0,
           90 => Rotation::R90,
           180 => Rotation::R180,
           270 => Rotation::R270,
           _ => panic!("Invalid rotation substraction result, {} - {}", a, b),
        }

    }
}

impl Tile {
    fn new(name: String, texture: TextureHandle) -> Self{
        Self {
            name,
            texture,
            connections: PatternConnections::new(),
        }
    }
}

impl CommonPattern {
    fn new(name: String) -> Self {
        Self {
            name,
            connections: PatternConnections::new(),
        }
    }
}

impl PatternConnections {
    fn new() -> Self {
        Self {
            top: Vec::new(),
            right: Vec::new(),
            left: Vec::new(),
            bottom: Vec::new(),
        }
    }
}

impl TileRulesEditorApp {
    fn new(tile_path: &str, tile_set_name: &str, symbols_path: &str) -> Self {
        Self {
            tile_path: tile_path.to_string(),
            tile_set_name: tile_set_name.to_string(),
            tiles: HashMap::new(),
            symbols_path: symbols_path.to_string(),
            symbols: HashMap::new(),
            selected: None,
            selection_state: SelectionState::Normal,
            selected_plus: None,
            tile_zoom: 3.0,
            common_patterns: HashMap::new(),
        }
    }

    fn add_pattern(&mut self) {
        let base = "Unnamed";
        let mut name = base.to_string();
        let mut counter = 1;
        while self.common_patterns.contains_key(&name) {
            name = format!("{} {}", base, counter);
            counter += 1;
        }

        self.common_patterns
            .insert(name.clone(), Rc::new(RefCell::new(CommonPattern::new(name))));
    }

    fn selected(&self) -> Option<&Selection> {
        self.selected.as_ref()
    }

    fn selected_plus(&self) -> Option<&Selection> {
        self.selected_plus.as_ref()
    }

    fn load_tiles(&mut self, ctx: &egui::Context) {
        self.tiles.clear();

        let full_path = format!("{}/{}/", self.tile_path, self.tile_set_name);
        if let Ok(entries) = fs::read_dir(&full_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext == "png" || ext == "jpg" {
                            if let Ok(bytes) = fs::read(&path) {
                                if let Ok(image) = image::load_from_memory(&bytes) {
                                    let size = [image.width() as usize, image.height() as usize];
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        image.to_rgba8().as_flat_samples().as_slice(),
                                    );
                                    let texture = ctx.load_texture(
                                        path.file_name().unwrap().to_string_lossy(),
                                        color_image,
                                        egui::TextureOptions::default(),
                                    );

                                    let name = path.file_name().unwrap().to_string_lossy().to_string();

                                    self.tiles.insert(name.clone(), Rc::new(RefCell::new(Tile::new(name, texture))));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_symbols(&mut self, ctx: &egui::Context) {
        self.symbols.clear();

        let full_path = &self.symbols_path;
        if let Ok(entries) = fs::read_dir(&full_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext == "png" || ext == "jpg" {
                            if let Ok(bytes) = fs::read(&path) {
                                if let Ok(image) = image::load_from_memory(&bytes) {
                                    let size = [image.width() as usize, image.height() as usize];
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        image.to_rgba8().as_flat_samples().as_slice(),
                                    );
                                    let texture = ctx.load_texture(
                                        path.file_name().unwrap().to_string_lossy(),
                                        color_image,
                                        egui::TextureOptions::default(),
                                    );

                                    let name =
                                        path.file_stem().unwrap().to_string_lossy().to_string();

                                    self.symbols.insert(name, texture);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Load neighbour rules from a JSON file produced by the editor.
    /// * Ignores the `"connections"` section entirely.
    /// * Only looks at `"allow"` lists (skips `"include"` / `"exclude"` if they exist).
    pub fn load_rules_from_json(&mut self, path: &str) -> Result<(), String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        let root: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        fn rot_from_u16(d: u16) -> Rotation {
            match d {
                90 => Rotation::R90,
                180 => Rotation::R180,
                270 => Rotation::R270,
                _ => Rotation::R0,
            }
        }

        fn take_allow_side(side: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
            side.get("allow")?.as_array()
        }

        let tiles_obj = root
            .get("tiles")
            .and_then(|v| v.as_object())
            .ok_or_else(|| "Missing \"tiles\" object".to_string())?;

        for (tile_name, tile_val) in tiles_obj {
            let tile_rc_ref: Rc<RefCell<Tile>> = match self.tiles.get(tile_name) {
                Some(rc) => Rc::clone(rc),
                None => {
                    continue;
                }
            };

            println!("Found: {}", tile_name);

            let mut conns = PatternConnections::new();

            let push_entry = |side_vec: &mut Vec<PatternEntry>, neigh_name: &str, rot: Rotation| {
                if let Some(neigh_rc) = self.tiles.get(neigh_name) {
                    side_vec.push(PatternEntry::Tile {
                        tile: Rc::clone(neigh_rc),
                        rotation: rot,
                    });
                }
            };

            for (side_key, dest) in [
                ("top", &mut conns.top),
                ("right", &mut conns.right),
                ("left", &mut conns.left),
                ("bottom", &mut conns.bottom),
            ] {
                if let Some(arr) = tile_val.get(side_key).and_then(take_allow_side) {
                    for entry in arr {
                        if let (Some(name), Some(rot)) = (entry.get("name"), entry.get("rotation")) {
                            if let (Some(name_str), Some(rot_u)) = (name.as_str(), rot.as_u64()) {
                                push_entry(dest, name_str, rot_from_u16(rot_u as u16));
                            }
                        }
                    }
                }
            }

            let tile_rc: &RefCell<Tile> = tile_rc_ref.borrow();

            tile_rc.borrow_mut().connections = conns;
        }

        Ok(())
    }
}

impl eframe::App for TileRulesEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("settings_panel").show(ctx, |ui| {
            ui.heading("Tile Loader Settings");

            ui.horizontal(|ui| {
                ui.label("Tile Set Name:");
                egui::ComboBox::from_id_salt(&self.tile_set_name)
                    .selected_text(format!("📁 {}", &self.tile_set_name))
                    .show_ui(ui, |ui| {
                        for name in get_available_tile_sets(&self.tile_path).iter() {
                            ui.selectable_value(&mut self.tile_set_name, name.to_string(), name);
                        }
                    })
            });

            ui.horizontal(|ui| {
                ui.label("Tile Path:");
                ui.text_edit_singleline(&mut self.tile_path);
            });

            if ui.button("Load tiles").clicked() {
                self.load_tiles(ctx);
                let _ = self.load_rules_from_json("rules.json");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WFC Tile Rules Editor");

            ui.horizontal(|ui| {
                ui.label("Loaded Tiles:");

                for (name, tile_rc_ref) in self.tiles.iter() {
                    let (image_response, is_selected, tile_rc_ref) = {
                        let selected = self.selected();

                        let is_selected = matches!(
                            &selected,
                            Some(Selection::Tile(selected_tile_rc)) if Rc::ptr_eq(selected_tile_rc, tile_rc_ref)
                        );

                        let tile_rc: &RefCell<Tile> = tile_rc_ref.borrow();

                        (
                        {
                            let tile: &Tile = &tile_rc.borrow();

                            ui.add(
                                egui::ImageButton::new(&tile.texture)
                                    .frame(false)
                                    .selected(is_selected)
                                    .sense(egui::Sense::click()),
                            )

                        },
                            is_selected,
                            tile_rc_ref
                        )
                    };

                    let primary_clicked = image_response.clicked();

                    if primary_clicked {
                        match self.selection_state {
                            SelectionState::Inserting => {
                                if let Some(Selection::Insert(ref direction)) = self.selected_plus {

                                    if let Some(current_selected) = self.selected() {
                                        match current_selected {
                                            Selection::Tile(selected_tile_rc_ref) => {
                                                let selected_tile_rc: &RefCell<Tile> = selected_tile_rc_ref.borrow();
                                                let mut selected_tile: std::cell::RefMut<'_, Tile> = selected_tile_rc.borrow_mut();

                                                let connection_list = match direction {
                                                    Direction::Top => &mut selected_tile.connections.top,
                                                    Direction::Right => &mut selected_tile.connections.right,
                                                    Direction::Bottom => &mut selected_tile.connections.bottom,
                                                    Direction::Left => &mut selected_tile.connections.left,
                                                };
                                                if primary_clicked {
                                                    for rot in Rotation::iter() {
                                                        let exists = connection_list.iter().any(|entry|
                                                            matches!(entry,
                                                                PatternEntry::Tile { tile: existing, rotation: r }
                                                                if Rc::ptr_eq(existing, tile_rc_ref) && *r == rot
                                                            )
                                                        );
                                                        if exists {
                                                            continue;
                                                        }
                                                        connection_list.push(PatternEntry::Tile {
                                                            tile: Rc::clone(tile_rc_ref),
                                                            rotation: rot.clone(),
                                                        });

                                                        // Check that the inserted tile is not
                                                        // itself, so it doesn't try to duplicate
                                                        // and run into issues
                                                        if !Rc::ptr_eq(tile_rc_ref, selected_tile_rc_ref) {
                                                            let inserted_tile_rc: &RefCell<Tile> = tile_rc_ref.borrow();
                                                            let mut inserted_tile: std::cell::RefMut<'_, Tile> = inserted_tile_rc.borrow_mut();

                                                            let mirror = direction.clone().opposite();
                                                            let reflected_side = mirror.rotated_ccw_by(rot.clone());

                                                            let mirror_list = match reflected_side {
                                                                Direction::Top => &mut inserted_tile.connections.top,
                                                                Direction::Right => &mut inserted_tile.connections.right,
                                                                Direction::Left => &mut inserted_tile.connections.left,
                                                                Direction::Bottom => &mut inserted_tile.connections.bottom,
                                                            };

                                                            let new_rotation = Rotation::R0.rotated_ccw_by(rot.clone());

                                                            mirror_list.push(PatternEntry::Tile {
                                                                tile: Rc::clone(selected_tile_rc_ref),
                                                                rotation: new_rotation,
                                                            });
                                                        }

                                                        break;
                                                    }
                                                }
                                            }
                                            Selection::CommonPattern(selected_pattern_rc_ref) => {
                                            }
                                            Selection::Insert(_) => {

                                            }
                                        }
                                    }
                                }
                            }
                            SelectionState::Normal => {
                                self.selected = Some(Selection::Tile(tile_rc_ref.clone()));
                            }
                        };
                    }

                    if is_selected {
                        let rect = image_response.rect;
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.5, egui::Color32::YELLOW),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            });

            let zoom_label = format!("Tile View Zoom {:.1}x", self.tile_zoom);

            ui.add(
                egui::Slider::new(&mut self.tile_zoom, 1.0..=10.0)
                    .text(zoom_label),
            );

            ui.label("Common Patterns:");
            ui.horizontal(|ui| {
                let size = egui::Vec2::splat(40.0);
                for (name, pattern_rc_ref) in self.common_patterns.iter() {
                    let pattern_rc: &RefCell<CommonPattern> = pattern_rc_ref.borrow();
                    let pattern: &CommonPattern = &pattern_rc.borrow();

                    ui.label(format!("{}", &pattern.name));

                    let is_selected = matches!(self.selected(), Some(Selection::CommonPattern(selection_pattern_rc_ref)) if name == &pattern.name);

                    let image = egui::Image::new(&self.symbols["grey_plus"]).fit_to_exact_size(size);

                    let image_response = ui.add(
                        egui::ImageButton::new(image)
                            .frame(false)
                            .selected(is_selected)
                            .sense(egui::Sense::click()),
                    );

                    if image_response.clicked() {
                        self.selected = Some(Selection::CommonPattern(pattern_rc_ref.clone()));
                    }

                    if is_selected {
                        let rect = image_response.rect;
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(3.0, egui::Color32::YELLOW),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            });

            if ui.button("Add New Pattern").clicked() {
                self.add_pattern();
            }

            if ui.button("Save to JSON").clicked() {
                // `tiles` is your HashMap<String, Rc<RefCell<Tile>>>
                let mut tiles_obj = serde_json::Map::new();

                for (name, tile_rc_ref) in &self.tiles {
                    let tile_rc: &RefCell<Tile> = tile_rc_ref.borrow();
                    let tile: &Tile = &tile_rc.borrow();

                    let tile_json = json!({
                        "top":    side_to_json(&tile.connections.top),
                        "right":  side_to_json(&tile.connections.right),
                        "left":   side_to_json(&tile.connections.left),
                        "bottom": side_to_json(&tile.connections.bottom),
                    });

                    tiles_obj.insert(name.clone(), tile_json);
                }

                let root = json!({"connections": {}, "tiles": tiles_obj });

                let output_file = "rules.json";

                match std::fs::write(&output_file, serde_json::to_string_pretty(&root).unwrap()) {
                    Ok(_)  => eprintln!("JSON saved to {}", output_file),
                    Err(e) => eprintln!("JSON save failed: {e:?}"),
                }
            }

            let selected_tile_rc = match &self.selected {
                Some(Selection::Tile(tile_rc_ref)) => Some(tile_rc_ref),
                _ => None,
            };

            if let Some(tile_rc_ref) = selected_tile_rc {
                // BEFORE building the UI
                let mut clicked: Option<(usize, usize, egui::PointerButton)> = None;
                let ( top_len, bottom_len, left_len, right_len ) = {
                    let tile_rc: &RefCell<Tile> = tile_rc_ref.borrow();
                    let tile: &Tile = &tile_rc.borrow();

                    let top_len = tile.connections.top.len();
                    let bottom_len = tile.connections.bottom.len();
                    let left_len = tile.connections.left.len();
                    let right_len = tile.connections.right.len();

                    let rows = top_len + 3 + bottom_len; // center already counts as 1
                    let cols = left_len + 3 + right_len; // center already counts as 1

                    let center_x = left_len + 1;
                    let center_y = top_len + 1;

                    let mut grid = vec![vec![Cell { kind: CellKind::Empty }; cols]; rows];

                    // Place the center tile
                    grid[center_y][center_x] = Cell { kind: CellKind::Tile{ texture: tile.texture.clone(), rotation: Rotation::R0 }};

                    // Fill top connections
                    for (i, pattern_entry) in tile.connections.top.iter().enumerate() {
                        let y = center_y - (i + 1);
                        let (texture, rotation) = get_texture_from_entry(pattern_entry);

                        grid[y][center_x] = Cell {
                            kind: CellKind::Tile { texture, rotation },
                        };
                    }

                    // Place top plus
                    grid[0][center_x] = Cell { kind: CellKind::Plus };

                    // Fill bottom connections
                    for (i, pattern_entry) in tile.connections.bottom.iter().enumerate() {
                        let y = center_y + (i + 1);
                        let (texture, rotation) = get_texture_from_entry(pattern_entry);

                        grid[y][center_x] = Cell {
                            kind: CellKind::Tile { texture, rotation },
                        };
                    }

                    // Place bottom plus
                    grid[rows - 1][center_x] = Cell { kind: CellKind::Plus };

                    // Fill left connections
                    for (i, pattern_entry) in tile.connections.left.iter().enumerate() {
                        let x = center_x - (i + 1);
                        let (texture, rotation) = get_texture_from_entry(pattern_entry);

                        grid[center_y][x] = Cell {
                            kind: CellKind::Tile { texture, rotation },
                        };
                    }

                    // Place left plus
                    grid[center_y][0] = Cell { kind: CellKind::Plus };

                    // Fill right connections
                    for (i, pattern_entry) in tile.connections.right.iter().enumerate() {
                        let x = center_x + (i + 1);
                        let (texture, rotation) = get_texture_from_entry(pattern_entry);

                        grid[center_y][x] = Cell {
                            kind: CellKind::Tile { texture, rotation },
                        };
                    }

                    // Place right plus
                    grid[center_y][cols - 1] = Cell { kind: CellKind::Plus };

                    let available_size = ui.available_size();
                    let size = tile.texture.size_vec2() * self.tile_zoom;

                    let grid_width = cols as f32 * size.x;
                    let grid_height = rows as f32 * size.y;
                    let grid_size = egui::Vec2::new(grid_width, grid_height);

                    let offset = (available_size - grid_size) * 0.5;

                    let rect = egui::Rect::from_min_size(
                        ui.min_rect().min + offset,
                        grid_size,
                    );

                    let ui_builder = egui::UiBuilder::new()
                        .max_rect(rect); // ✅

                    ui.allocate_new_ui(ui_builder, |ui| {
                        ui.vertical(|ui| {
                            for (grid_y, row) in grid.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    for (grid_x, cell) in row.iter().enumerate() {
                                        match &cell.kind {
                                            CellKind::Tile { texture, rotation } => {
                                                // reserve layout space and get the response
                                                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

                                                // draw the tile
                                                draw_texture_rotated(ui, texture, rect.min, size, rotation.clone());

                                                // highlight the centre tile
                                                if grid_x == center_x && grid_y == center_y {
                                                    ui.painter().rect_stroke(
                                                        rect,
                                                        0.0,
                                                        egui::Stroke::new(2.5, egui::Color32::YELLOW),
                                                        egui::StrokeKind::Outside,
                                                    );
                                                }

                                                // rotate on left-click, but NOT if it is the centre tile
                                                if response.clicked_by(egui::PointerButton::Primary)
                                                && !(grid_x == center_x && grid_y == center_y)
                                                {
                                                    println!("Clicked at ({}, {})", grid_x, grid_y);
                                                    clicked = Some((grid_x, grid_y, egui::PointerButton::Primary));
                                                }
                                                // remove on right_click, but NOT if it is the centre tile
                                                if response.clicked_by(egui::PointerButton::Secondary)
                                                && !(grid_x == center_x && grid_y == center_y)
                                                {
                                                    println!("Clicked at ({}, {})", grid_x, grid_y);
                                                    clicked = Some((grid_x, grid_y, egui::PointerButton::Secondary));
                                                }
                                            }
                                            CellKind::Plus => {
                                                let image = egui::Image::new(&self.symbols["grey_plus"])
                                                    .fit_to_exact_size(size);

                                                let image_response = ui.add(
                                                    egui::ImageButton::new(image)
                                                        .frame(false)
                                                        .sense(egui::Sense::click())
                                                );

                                                if image_response.clicked() {
                                                    if let Some(direction) = plus_direction(grid_x, grid_y, center_x, center_y, rows, cols) {
                                                        match &self.selected_plus {
                                                            Some(Selection::Insert(current_direction)) if *current_direction == direction => {
                                                                self.selected_plus = None;
                                                                self.selection_state = SelectionState::Normal;
                                                            }
                                                            _ => {
                                                                self.selection_state = SelectionState::Inserting;
                                                                self.selected_plus = Some(Selection::Insert(direction));
                                                            }
                                                        }
                                                    }
                                                }

                                                if let Some(Selection::Insert(insert_direction)) = &self.selected_plus {
                                                    if let Some(direction) = plus_direction(grid_x, grid_y, center_x, center_y, rows, cols) {
                                                        if *insert_direction == direction {
                                                            let rect = image_response.rect;
                                                            ui.painter().rect_stroke(
                                                                rect,
                                                                0.0,
                                                                egui::Stroke::new(2.0, egui::Color32::GREEN),
                                                                egui::StrokeKind::Outside,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            CellKind::Empty => {
                                                ui.add_sized(size, egui::Separator::default());
                                            }
                                        }
                                    }
                                });
                            }
                        });
                    });
                    (
                        top_len,
                        bottom_len,
                        left_len,
                        right_len,
                    )
                };
                {
                    if let Some((gx, gy, button)) = clicked {
                        let tile_rc: &RefCell<Tile> = tile_rc_ref.borrow();

                        let center_x = left_len + 1;
                        let center_y = top_len + 1;

                        // how far from the centre did we click?
                        let dx = gx as isize - center_x as isize;
                        let dy = gy as isize - center_y as isize;

                        // helper that rotates a PatternEntry in-place
                        let rotate_entry = |entry: &mut PatternEntry| {
                            if let PatternEntry::Tile { rotation, .. } = entry {
                                *rotation = rotation.clone().rotated_cw();
                            }
                        };

                        if dx == 0 && dy < 0 {
                            // clicked *above* centre → top list, index = |dy| – 1
                            let idx = (-dy - 1) as usize;
                            if button == egui::PointerButton::Primary {
                                rotate_connection_handler(tile_rc_ref, Direction::Top, idx);
                            } else if button == egui::PointerButton::Secondary {
                                remove_connection_handler(tile_rc_ref, Direction::Top, idx);
                            }
                        } else if dx == 0 && dy > 0 {
                            // below
                            let idx = (dy - 1) as usize;
                            if button == egui::PointerButton::Primary {
                                rotate_connection_handler(tile_rc_ref, Direction::Bottom, idx);
                            } else if button == egui::PointerButton::Secondary {
                                remove_connection_handler(tile_rc_ref, Direction::Bottom, idx);
                            }
                        } else if dy == 0 && dx < 0 {
                            // left
                            let idx = (-dx - 1) as usize;
                            if button == egui::PointerButton::Primary {
                                rotate_connection_handler(tile_rc_ref, Direction::Left, idx);
                            } else if button == egui::PointerButton::Secondary {
                                remove_connection_handler(tile_rc_ref, Direction::Left, idx);
                            }
                        } else if dy == 0 && dx > 0 {
                            // right
                            let idx = (dx - 1) as usize;
                            if button == egui::PointerButton::Primary {
                                rotate_connection_handler(tile_rc_ref, Direction::Right, idx);
                            } else if button == egui::PointerButton::Secondary {
                                remove_connection_handler(tile_rc_ref, Direction::Right, idx);
                            }
                        }
                    }
                }
            }
        });
    }
}


/// Call this whenever you rotate a neighbour tile in the grid.
/// `side` is the side on *selected* you clicked (Top/Right/…)
/// `idx`   is the entry index inside that side-vector.
fn rotate_connection_handler(
    selected_rc_ref: &Rc<RefCell<Tile>>,
    side: Direction,
    idx: usize,
) {

    let mut chosen_rot = None;

    // ─── Pre-check if a free rotation exists ──────────────────────
    {
        let a_selected_ref: &RefCell<Tile> = selected_rc_ref.borrow();
        let a_selected: std::cell::Ref<'_, Tile> = a_selected_ref.borrow();

        let entry = match side {
            Direction::Top    => a_selected.connections.top.get(idx),
            Direction::Right  => a_selected.connections.right.get(idx),
            Direction::Bottom => a_selected.connections.bottom.get(idx),
            Direction::Left   => a_selected.connections.left.get(idx),
        };

        let (tile_rc_ref, rotation) = match entry {
            Some(PatternEntry::Tile { tile, rotation }) => (tile, rotation),
            _ => {
                println!("Clicked a CommonPattern or invalid entry, no rotation.");
                return;
            }
        };

        let mut simulated_rot = rotation.clone().rotated_cw(); // start at +90°

        for attempt in 0..3 { // try 90°, 180°, 270°
            let expected_rot = Rotation::R0.rotated_cw_by(simulated_rot.clone());

            let already_exists = [
                &a_selected.connections.top,
                &a_selected.connections.right,
                &a_selected.connections.bottom,
                &a_selected.connections.left,
            ]
            .iter()
            .any(|vec| {
                vec.iter().any(|entry| match entry {
                    PatternEntry::Tile { tile: existing, rotation: existing_rot } =>
                        Rc::ptr_eq(existing, tile_rc_ref) && *existing_rot == expected_rot,
                    _ => false,
                })
            });

            println!(
                "[Pre-check Attempt {attempt}] Trying simulated_rot={:?}, expected_rot={:?} => {}",
                simulated_rot,
                expected_rot,
                if already_exists { "Already exists, rotating further..." } else { "Free, will rotate!" }
            );

            if !already_exists {
                chosen_rot = Some(simulated_rot.clone());
                break; // found a valid free rotation
            }

            simulated_rot = simulated_rot.rotated_cw();
        }
    }

    let chosen_rot = match chosen_rot {
       Some(rot) => rot,
        None => {
            println!("No free rotations available, skipping rotation.");
            return;
        }
    };

    // ─────────────────────────────────────────────────────────────
    //
    // ─── 1. mutate the selected tile (A) ──────────────────────
    let (neighbour_rc_ref, old_rot) = {
        let a_selected_ref: &RefCell<Tile> = selected_rc_ref.borrow();
        let mut a_selected: std::cell::RefMut<'_, Tile> = a_selected_ref.borrow_mut();
        let entry = match side {
            Direction::Top    => &mut a_selected.connections.top[idx],
            Direction::Right  => &mut a_selected.connections.right[idx],
            Direction::Bottom => &mut a_selected.connections.bottom[idx],
            Direction::Left   => &mut a_selected.connections.left[idx],
        };

        if let PatternEntry::Tile { tile: b_rc, rotation } = entry {
            let original_b_rotation = rotation.clone();
            *rotation = chosen_rot.clone();
            println!("Selected tile rotated: {:?} → {:?}", original_b_rotation.clone(), original_b_rotation.clone().rotated_cw());
            (Rc::clone(&b_rc), original_b_rotation)          // pass B + new rot out
        } else {
            return;                                     // clicked a CommonPattern → nothing to do
        }
    };

    {
        let b_selected_ref: &RefCell<Tile> = neighbour_rc_ref.borrow();
        let b_selected: RefMut<'_, Tile> = b_selected_ref.borrow_mut();

        println!("Neighbour B connections:");
        for (dir_name, vec) in [
            ("Top", &b_selected.connections.top),
            ("Right", &b_selected.connections.right),
            ("Bottom", &b_selected.connections.bottom),
            ("Left", &b_selected.connections.left),
        ] {
            for (i, entry) in vec.iter().enumerate() {
                if let PatternEntry::Tile { tile, rotation } = entry {
                    let pointer = Rc::ptr_eq(tile, selected_rc_ref);
                    println!("  {dir_name}[{i}]: tile matches selected? {pointer}, rotation={:?}", rotation);
                }
            }
        }
    }

    //------------------------------------------------------------
    // 2  Find which side on B currently holds (A, old_rot)
    //------------------------------------------------------------
    let mirror = side.opposite();
    let reflected_side = mirror.clone().rotated_ccw_by(old_rot.clone());
    let new_side = reflected_side.clone().rotated_cw_by(old_rot - chosen_rot);

    println!("mirror: {:?}, reflected_side: {:?}, new_side: {:?}", mirror.clone(), reflected_side.clone(), new_side.clone());
    println!("old_rot: {:?}, chosen_rot: {:?}", old_rot.clone(), chosen_rot.clone());

    let b_selected_ref: &RefCell<Tile> = neighbour_rc_ref.borrow();
    let mut b_selected: RefMut<'_, Tile> = b_selected_ref.borrow_mut();

    let src_vec = match reflected_side {
        Direction::Top    => &mut b_selected.connections.top,
        Direction::Right  => &mut b_selected.connections.right,
        Direction::Bottom => &mut b_selected.connections.bottom,
        Direction::Left   => &mut b_selected.connections.left,
    };

    let expected_rot = Rotation::R0.rotated_ccw_by(old_rot.clone());

    println!("Expected_rot: {:?}", expected_rot.clone());

    // Find the position of the entry we want to move
    let pos = src_vec.iter()
        .position(|entry| match entry {
            PatternEntry::Tile { tile, rotation } =>
                Rc::ptr_eq(tile, selected_rc_ref) && *rotation == expected_rot,
            _ => false,
        });

    let pos = match pos {
        Some(p) => p,
        None => {
            eprintln!("Could not find matching entry to rotate!");
            return;
        }
    };

    // take the entry out
    let mut entry = src_vec.remove(pos);

    // ➋ update its rotation
    if let PatternEntry::Tile { rotation, .. } = &mut entry {
        let b_rot = rotation.clone().rotated_cw_by(old_rot - chosen_rot);
        *rotation = b_rot.clone();
        println!("Moving entry from {reflected_side:?} to {new_side:?}, setting rotation to {:?}", b_rot);
    }

    let target_vec = match new_side {
        Direction::Top    => &mut b_selected.connections.top,
        Direction::Right  => &mut b_selected.connections.right,
        Direction::Bottom => &mut b_selected.connections.bottom,
        Direction::Left   => &mut b_selected.connections.left,
    };

    // ➍ push it there (deduplicated)
    if !target_vec.iter().any(|e|
        matches!(e,
            PatternEntry::Tile { tile, rotation }
            if Rc::ptr_eq(tile, selected_rc_ref) && *rotation == *match &entry { PatternEntry::Tile { rotation, .. } => rotation, _ => &Rotation::R0 }
        )
    ) {
        target_vec.push(entry);
    }
}


/// Call this whenever you *remove* a neighbour tile from the grid.
/// `side` is the side on *selected* you clicked  (Top / Right / …)
/// `idx`  is the entry-index inside that side-vector.
fn remove_connection_handler(
    selected_rc_ref: &Rc<RefCell<Tile>>,
    side: Direction,
    idx: usize,
) {
    // ── 1. take the entry out of   A   ─────────────────────────────
    let (neighbour_rc_ref, old_rot) = {
        // mutable borrow of A *only* for the removal itself
        let a_selected_ref: &RefCell<Tile> = selected_rc_ref.borrow();
        let mut a_selected: std::cell::RefMut<'_, Tile> = a_selected_ref.borrow_mut();

        let list = match side {
            Direction::Top    => &mut a_selected.connections.top,
            Direction::Right  => &mut a_selected.connections.right,
            Direction::Bottom => &mut a_selected.connections.bottom,
            Direction::Left   => &mut a_selected.connections.left,
        };

        // idx is guaranteed to exist (UI gives valid index)
        let removed = list.remove(idx);

        // if the removed entry was *not* a Tile, nothing else to do.
        let (tile_rc, rot) = match removed {
            PatternEntry::Tile { tile, rotation } => (tile, rotation),
            _ => return,
        };

        (tile_rc, rot)
    };

    // ── 2. remove mirrored entry from   B   ───────────────────────
    {
        let mirror_side     = side.opposite();           // which side of A faces B
        let reflected_side  = mirror_side
            .rotated_ccw_by(old_rot.clone());           // how B sees that side
        let expected_rot_on_b = Rotation::R0
            .rotated_ccw_by(old_rot.clone());           // rotation B stored for A

        let b_selected_ref: &RefCell<Tile> = neighbour_rc_ref.borrow();
        let mut b_selected: RefMut<'_, Tile> = b_selected_ref.borrow_mut();

        let list = match reflected_side {
            Direction::Top    => &mut b_selected.connections.top,
            Direction::Right  => &mut b_selected.connections.right,
            Direction::Bottom => &mut b_selected.connections.bottom,
            Direction::Left   => &mut b_selected.connections.left,
        };

        if let Some(pos) = list.iter().position(|entry| {
            matches!(
                entry,
                PatternEntry::Tile { tile, rotation }
                if Rc::ptr_eq(tile, selected_rc_ref) && *rotation == expected_rot_on_b
            )
        }) {
            list.remove(pos);
        }
    }
}

// ───────── helper used above ─────────
fn points_to_selected(entry: &PatternEntry, selected: &Rc<RefCell<Tile>>) -> bool {
    matches!(entry,
        PatternEntry::Tile { tile, .. }
        if Rc::ptr_eq(tile, selected)
    )
}

fn get_texture_from_entry(entry: &PatternEntry) -> (egui::TextureHandle, Rotation) {
    match entry {
        PatternEntry::Tile { tile: tile_rc_ref, rotation } => {
            let tile_rc: &RefCell<Tile> = tile_rc_ref.borrow();
            let texture = tile_rc.borrow().texture.clone();
            (texture, rotation.clone())
        },
        PatternEntry::CommonPattern(_pattern_rc) => {
            panic!("CommonPattern drawing not handled yet")
        }
    }
}

fn rotation_to_uv(rotation: Rotation) -> (egui::Pos2, egui::Pos2) {
    use egui::pos2;
    match rotation {
        Rotation::R0 => (pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Rotation::R90 => (pos2(1.0, 0.0), pos2(0.0, 1.0)),
        Rotation::R180 => (pos2(1.0, 1.0), pos2(0.0, 0.0)),
        Rotation::R270 => (pos2(0.0, 1.0), pos2(1.0, 0.0)),
    }
}

fn draw_texture_rotated(
    ui: &mut egui::Ui,
    texture: &egui::TextureHandle,
    pos: egui::Pos2,
    size: egui::Vec2,
    rotation: Rotation,          // R0 | R90 | R180 | R270
) {
    let rect   = egui::Rect::from_min_size(pos, size);
    let center = rect.center();
    let hs     = rect.size() * 0.5;                      // half-size

    // square corners around (0,0)
    let corners = [
        egui::vec2(-hs.x, -hs.y), // TL
        egui::vec2( hs.x, -hs.y), // TR
        egui::vec2( hs.x,  hs.y), // BR
        egui::vec2(-hs.x,  hs.y), // BL
    ];

    const UVS: [egui::Pos2; 4] = [
        egui::pos2(0.0, 0.0),   // TL
        egui::pos2(1.0, 0.0),   // TR
        egui::pos2(1.0, 1.0),   // BR
        egui::pos2(0.0, 1.0),   // BL
    ];

    // screen Y-axis points downward, so clockwise 90° is (-y, x)
    let rotate = |v: egui::Vec2, rot: &Rotation| -> egui::Vec2 {
        match rot {
            Rotation::R0   => v,
            Rotation::R90  => egui::vec2(-v.y,  v.x),
            Rotation::R180 => egui::vec2(-v.x, -v.y),
            Rotation::R270 => egui::vec2( v.y, -v.x),
        }
    };

    let mut mesh = egui::Mesh::default();
    mesh.texture_id = texture.id();

    for i in 0..4 {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + rotate(corners[i], &rotation), // moved corner
            uv:  UVS[i],                                 // **unchanged** UV
            color: egui::Color32::WHITE,
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);

    ui.painter().add(egui::Shape::mesh(mesh));
}

fn plus_direction(grid_x: usize, grid_y: usize, center_x: usize, center_y: usize, rows: usize, cols: usize) -> Option<Direction> {
    if grid_x == center_x && grid_y == 0 {
        Some(Direction::Top)
    } else if grid_x == center_x && grid_y == rows - 1 {
        Some(Direction::Bottom)
    } else if grid_y == center_y && grid_x == 0 {
        Some(Direction::Left)
    } else if grid_y == center_y && grid_x == cols - 1 {
        Some(Direction::Right)
    } else {
        None
    }
}

fn get_available_tile_sets(tile_path: &str) -> Vec<String> {
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

/// Convert one side (top/right/left/bottom) into a JSON array,
/// silently skipping every `CommonPattern`.
fn side_to_json(side: &[PatternEntry]) -> Value {
    let allow: Vec<Value> = side.iter()
        .filter_map(|p| {
            if let PatternEntry::Tile { tile, rotation } = p {
                let tile_rc: &RefCell<Tile> = tile.borrow();
                let tile: &Tile = &tile_rc.borrow();
                Some(json!(Entry {
                    name: &tile.name,
                    rotation: rotation.clone().degrees()
                }))
            } else {
                None // ignore CommonPattern
            }
        })
        .collect();

    json!({ "allow": allow })
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WFC Rules Editor",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let tile_path = "main_wfc/src/assets/image/";
            let tile_set_name = "circuit";

            let symbols_path = "wfc_tool/src/assets/image/symbols/";

            let mut app = TileRulesEditorApp::new(tile_path, tile_set_name, symbols_path);
            app.load_tiles(&cc.egui_ctx);
            let _ = app.load_rules_from_json("rules.json");
            app.load_symbols(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
