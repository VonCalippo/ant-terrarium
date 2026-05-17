# Core World & Rendering — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the 2D grid world, terrain physics, rendering, UI, and persistence for the Ant Terrarium Simulator.

**Architecture:** Two Rust crates in a Cargo workspace — `ant_simulation` (pure Rust, no Bevy) handles grid, terrain physics, tick loop, and persistence; `ant_renderer` (Bevy plugin) consumes snapshots and renders the terrarium with HUD and camera controls.

**Tech Stack:** Rust stable, Bevy 0.15 (bevy_sprite, bevy_ui, bevy_input), serde + bincode

---

## File Map

```
ant/
├── Cargo.toml                          # workspace: members = ["crates/*"]
├── src/main.rs                         # binary entry point
├── crates/
│   ├── ant_simulation/
│   │   ├── Cargo.toml                  # deps: serde, bincode, rand
│   │   └── src/
│   │       ├── lib.rs                  # pub mod grid, terrain, tick, snapshot, persistence
│   │       ├── grid.rs                 # Material, Cell, GridPos, Direction, Grid
│   │       ├── terrain.rs              # DigState, TerrainEvent, stability update, digging
│   │       ├── tick.rs                 # Speed, Simulation, TickResult
│   │       ├── snapshot.rs            # Snapshot, CellSnapshot, Command
│   │       └── persistence.rs          # SaveFile, save(), load()
│   └── ant_renderer/
│       ├── Cargo.toml                  # deps: bevy 0.15 (sprite, ui, input), ant_simulation
│       └── src/
│           ├── lib.rs                  # pub mod {app, camera, sprites, hud, input, assets}
│           ├── app.rs                  # TerrariumPlugin, system sets
│           ├── camera.rs               # Camera2D setup, zoom, pan, snap
│           ├── sprites.rs              # Grid rendering, sprite spawning/updating, glass overlay
│           ├── hud.rs                  # Bottom bar + side panel UI
│           ├── input.rs                # Mouse/keyboard → Command generation
│           └── assets.rs              # Color constants, initial world builder
```

---

## Phase 1: Workspace & Core Simulation Crate

### Task 1: Create Cargo workspace structure

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `crates/ant_simulation/Cargo.toml`
- Create: `crates/ant_simulation/src/lib.rs`

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { version = "0.15", default-features = false, features = ["bevy_sprite", "bevy_ui", "bevy_input", "bevy_window"] }
ant_simulation = { path = "crates/ant_simulation" }
ant_renderer = { path = "crates/ant_renderer" }
```

- [ ] **Step 2: Create minimal src/main.rs**

```rust
fn main() {
    ant_renderer::run();
}
```

- [ ] **Step 3: Create crates/ant_simulation/Cargo.toml**

```toml
[package]
name = "ant_simulation"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
bincode = "2"
rand = "0.8"
```

- [ ] **Step 4: Create crates/ant_simulation/src/lib.rs**

```rust
pub mod grid;
pub mod terrain;
pub mod tick;
pub mod snapshot;
pub mod persistence;
```

- [ ] **Step 5: Create placeholder module files**

Create empty files so the project compiles:
- `crates/ant_simulation/src/grid.rs` with `// Grid data model`
- `crates/ant_simulation/src/terrain.rs` with `// Terrain physics`
- `crates/ant_simulation/src/tick.rs` with `// Tick engine`
- `crates/ant_simulation/src/snapshot.rs` with `// Snapshot interface`
- `crates/ant_simulation/src/persistence.rs` with `// Save/load`

- [ ] **Step 6: Create placeholder ant_renderer crate**

Create `crates/ant_renderer/Cargo.toml`:
```toml
[package]
name = "ant_renderer"
version.workspace = true
edition.workspace = true

[dependencies]
bevy.workspace = true
ant_simulation.workspace = true
```

Create `crates/ant_renderer/src/lib.rs`:
```rust
pub fn run() {
    println!("Ant renderer placeholder");
}
```

- [ ] **Step 7: Verify workspace builds**

Run: `cargo build`
Expected: Compiles successfully with no warnings (or just dead_code warnings on empty modules).

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml src/main.rs crates/
git commit -m "feat: scaffold Cargo workspace with ant_simulation and ant_renderer crates"
```

---

### Task 2: Define Material enum and Cell struct

**Files:**
- Create: `crates/ant_simulation/src/grid.rs`

- [ ] **Step 1: Write the grid module with Material, Cell, PheromoneLayer, GridPos, Direction**

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    Air,
    LooseDirt,
    Dirt,
    WetDirt,
    Sand,
    Stone,
    Water,
    Food,
    OrganicWaste,
    Fungus,
    Egg,
    Larva,
}

impl Material {
    pub fn is_terrain(self) -> bool {
        matches!(self, Material::Dirt | Material::LooseDirt | Material::WetDirt | Material::Sand)
    }

    pub fn is_solid(self) -> bool {
        !matches!(self, Material::Air | Material::Water)
    }

    pub fn is_diggable(self) -> bool {
        matches!(self, Material::Dirt | Material::WetDirt | Material::Sand | Material::LooseDirt)
    }

    pub fn dig_ticks(self) -> Option<u8> {
        match self {
            Material::Dirt => Some(1),
            Material::WetDirt => Some(2),
            Material::Sand => Some(0),
            Material::LooseDirt => Some(0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PheromoneLayer {
    pub food: u8,
    pub home: u8,
    pub danger: u8,
    pub dig: u8,
    pub queen: u8,
    pub death: u8,
    pub waste: u8,
}

impl Default for PheromoneLayer {
    fn default() -> Self {
        Self { food: 0, home: 0, danger: 0, dig: 0, queen: 0, death: 0, waste: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPos {
    pub x: u16,
    pub y: u16,
}

impl GridPos {
    pub fn new(x: u16, y: u16) -> Self { Self { x, y } }

    pub fn neighbor(self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::N => self.y.checked_sub(1).map(|y| GridPos { y, ..self }),
            Direction::S => Some(GridPos { y: self.y + 1, ..self }),
            Direction::E => Some(GridPos { x: self.x + 1, ..self }),
            Direction::W => self.x.checked_sub(1).map(|x| GridPos { x, ..self }),
            Direction::NE => self.neighbor(Direction::N).and_then(|p| p.neighbor(Direction::E)),
            Direction::NW => self.neighbor(Direction::N).and_then(|p| p.neighbor(Direction::W)),
            Direction::SE => self.neighbor(Direction::S).and_then(|p| p.neighbor(Direction::E)),
            Direction::SW => self.neighbor(Direction::S).and_then(|p| p.neighbor(Direction::W)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N, S, E, W, NE, NW, SE, SW,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::N, Direction::S, Direction::E, Direction::W,
        Direction::NE, Direction::NW, Direction::SE, Direction::SW,
    ];

    pub const CARDINAL: [Direction; 4] = [
        Direction::N, Direction::S, Direction::E, Direction::W,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub material: Material,
    pub humidity: u8,
    pub temperature: i16,
    pub stability: u8,
    pub pheromones: PheromoneLayer,
    pub organic_matter: u8,
}

impl Cell {
    pub fn new(material: Material) -> Self {
        let (stability, humidity) = match material {
            Material::Stone => (255, 0),
            Material::Dirt => (128, 30),
            Material::WetDirt => (128, 180),
            Material::Sand => (64, 5),
            Material::LooseDirt => (64, 20),
            Material::Air => (0, 0),
            Material::Water => (0, 255),
            Material::Food => (0, 60),
            _ => (0, 0),
        };
        Self {
            material,
            humidity,
            temperature: 220, // 22.0°C default
            stability,
            pheromones: PheromoneLayer::default(),
            organic_matter: 0,
        }
    }
}
```

- [ ] **Step 2: Write tests for Material methods**

Add to bottom of `grid.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_is_terrain() {
        assert!(Material::Dirt.is_terrain());
        assert!(Material::Sand.is_terrain());
        assert!(Material::LooseDirt.is_terrain());
        assert!(Material::WetDirt.is_terrain());
        assert!(!Material::Air.is_terrain());
        assert!(!Material::Stone.is_terrain());
        assert!(!Material::Water.is_terrain());
    }

    #[test]
    fn test_material_is_solid() {
        assert!(Material::Dirt.is_solid());
        assert!(Material::Stone.is_solid());
        assert!(!Material::Air.is_solid());
        assert!(!Material::Water.is_solid());
    }

    #[test]
    fn test_material_diggable() {
        assert!(Material::Dirt.is_diggable());
        assert!(Material::Sand.is_diggable());
        assert!(Material::WetDirt.is_diggable());
        assert!(!Material::Stone.is_diggable());
        assert!(!Material::Air.is_diggable());
    }

    #[test]
    fn test_material_dig_ticks() {
        assert_eq!(Material::Dirt.dig_ticks(), Some(1));
        assert_eq!(Material::WetDirt.dig_ticks(), Some(2));
        assert_eq!(Material::Sand.dig_ticks(), Some(0));
        assert_eq!(Material::Stone.dig_ticks(), None);
    }

    #[test]
    fn test_gridpos_neighbor_cardinals() {
        let pos = GridPos::new(5, 5);
        assert_eq!(pos.neighbor(Direction::N), Some(GridPos::new(5, 4)));
        assert_eq!(pos.neighbor(Direction::S), Some(GridPos::new(5, 6)));
        assert_eq!(pos.neighbor(Direction::E), Some(GridPos::new(6, 5)));
        assert_eq!(pos.neighbor(Direction::W), Some(GridPos::new(4, 5)));
    }

    #[test]
    fn test_gridpos_neighbor_boundary() {
        let pos = GridPos::new(0, 0);
        assert_eq!(pos.neighbor(Direction::N), None);
        assert_eq!(pos.neighbor(Direction::W), None);
        assert_eq!(pos.neighbor(Direction::NW), None);
    }

    #[test]
    fn test_pheromone_layer_default() {
        let p = PheromoneLayer::default();
        assert_eq!(p.food, 0);
        assert_eq!(p.home, 0);
        assert_eq!(p.waste, 0);
    }

    #[test]
    fn test_cell_new_defaults() {
        let dirt = Cell::new(Material::Dirt);
        assert_eq!(dirt.material, Material::Dirt);
        assert_eq!(dirt.stability, 128);
        assert_eq!(dirt.humidity, 30);

        let air = Cell::new(Material::Air);
        assert_eq!(air.stability, 0);
        assert_eq!(air.humidity, 0);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All 8 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_simulation/src/grid.rs
git commit -m "feat: add Material, Cell, GridPos, Direction, PheromoneLayer types"
```

---

### Task 3: Define Grid struct with world generation

**Files:**
- Modify: `crates/ant_simulation/src/grid.rs` (add Grid struct and methods)

- [ ] **Step 1: Add Grid struct and core methods to grid.rs**

Add after existing types in `grid.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        let len = width as usize * height as usize;
        Self {
            width,
            height,
            cells: vec![Cell::new(Material::Air); len],
        }
    }

    pub fn index(&self, pos: GridPos) -> usize {
        pos.y as usize * self.width as usize + pos.x as usize
    }

    pub fn contains(&self, pos: GridPos) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    pub fn get(&self, pos: GridPos) -> Option<&Cell> {
        if self.contains(pos) {
            Some(&self.cells[self.index(pos)])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, pos: GridPos) -> Option<&mut Cell> {
        if self.contains(pos) {
            let idx = self.index(pos);
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }

    pub fn set_material(&mut self, pos: GridPos, material: Material) -> bool {
        if let Some(cell) = self.get_mut(pos) {
            cell.material = material;
            cell.stability = Cell::new(material).stability;
            if material == Material::Air {
                cell.humidity = 0;
            }
            true
        } else {
            false
        }
    }

    pub fn cell_below(&self, pos: GridPos) -> Option<&Cell> {
        self.get(pos.neighbor(Direction::S)?)
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = GridPos> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| GridPos::new(x, y))
        })
    }

    pub fn surface_y(&self) -> u16 {
        // first row (lowest y) where material is Air going top-down
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(GridPos::new(x, y)).map(|c| c.material == Material::Air).unwrap_or(false) {
                    return y;
                }
            }
        }
        self.height
    }
}
```

- [ ] **Step 2: Write tests for Grid**

Add to test module in `grid.rs`:

```rust
#[test]
fn test_grid_new_correct_size() {
    let grid = Grid::new(128, 96);
    assert_eq!(grid.width, 128);
    assert_eq!(grid.height, 96);
    assert_eq!(grid.cells.len(), 128 * 96);
    // All cells should be Air
    assert!(grid.cells.iter().all(|c| c.material == Material::Air));
}

#[test]
fn test_grid_index() {
    let grid = Grid::new(3, 3);
    assert_eq!(grid.index(GridPos::new(0, 0)), 0);
    assert_eq!(grid.index(GridPos::new(2, 0)), 2);
    assert_eq!(grid.index(GridPos::new(0, 1)), 3);
    assert_eq!(grid.index(GridPos::new(2, 2)), 8);
}

#[test]
fn test_grid_contains() {
    let grid = Grid::new(10, 10);
    assert!(grid.contains(GridPos::new(0, 0)));
    assert!(grid.contains(GridPos::new(9, 9)));
    assert!(!grid.contains(GridPos::new(10, 0)));
    assert!(!grid.contains(GridPos::new(0, 10)));
}

#[test]
fn test_grid_get_and_set() {
    let mut grid = Grid::new(10, 10);
    grid.set_material(GridPos::new(3, 3), Material::Dirt);

    let cell = grid.get(GridPos::new(3, 3)).unwrap();
    assert_eq!(cell.material, Material::Dirt);
    assert_eq!(cell.stability, 128);

    assert!(grid.get(GridPos::new(100, 100)).is_none());
}

#[test]
fn test_grid_cell_below() {
    let mut grid = Grid::new(10, 10);
    grid.set_material(GridPos::new(5, 4), Material::Dirt);
    let below = grid.cell_below(GridPos::new(5, 3));
    assert!(below.is_some());
    assert_eq!(below.unwrap().material, Material::Dirt);
}

#[test]
fn test_grid_surface_y_all_air() {
    let grid = Grid::new(10, 10);
    assert_eq!(grid.surface_y(), 0); // all air, surface at top
}

#[test]
fn test_grid_iter_positions() {
    let grid = Grid::new(4, 3);
    let positions: Vec<_> = grid.iter_positions().collect();
    assert_eq!(positions.len(), 12);
    assert_eq!(positions[0], GridPos::new(0, 0));
    assert_eq!(positions[11], GridPos::new(3, 2));
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All 15 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_simulation/src/grid.rs
git commit -m "feat: add Grid struct with world generation and spatial queries"
```

---

### Task 4: Define terrain physics types and digging logic

**Files:**
- Create: `crates/ant_simulation/src/terrain.rs`

- [ ] **Step 1: Write terrain.rs with DigState, TerrainEvent, and digging functions**

```rust
use crate::grid::{Grid, GridPos, Material};

#[derive(Debug, Clone)]
pub struct DigState {
    pub target: GridPos,
    pub ticks_remaining: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainEvent {
    DigProgress { pos: GridPos, remaining: u8 },
    DigComplete { pos: GridPos },
    Collapse { pos: GridPos, from: Material, to: Material },
    CollapseChain { positions: Vec<GridPos> },
}

const STABILITY_DECAY: u8 = 4;      // per tick when unsupported
const STABILITY_RECOVERY: u8 = 2;   // per tick when supported

pub fn process_digging(grid: &mut Grid, pending: &mut Vec<DigState>) -> Vec<TerrainEvent> {
    let mut events = Vec::new();
    let mut completed = Vec::new();

    for (i, dig) in pending.iter_mut().enumerate() {
        if dig.ticks_remaining == 0 {
            // Instant dig (Sand, LooseDirt)
            grid.set_material(dig.target, Material::Air);
            events.push(TerrainEvent::DigComplete { pos: dig.target });
            completed.push(i);
        } else {
            dig.ticks_remaining -= 1;
            if dig.ticks_remaining == 0 {
                // Check intermediate step
                let cell = grid.get(dig.target);
                if let Some(cell) = cell {
                    let next = match cell.material {
                        Material::WetDirt => Material::LooseDirt,
                        _ => Material::Air,
                    };
                    grid.set_material(dig.target, next);
                    if next == Material::Air {
                        events.push(TerrainEvent::DigComplete { pos: dig.target });
                    } else {
                        events.push(TerrainEvent::DigProgress { pos: dig.target, remaining: 0 });
                    }
                }
                completed.push(i);
            } else {
                events.push(TerrainEvent::DigProgress { pos: dig.target, remaining: dig.ticks_remaining });
            }
        }
    }

    // Remove completed digs (reverse order to preserve indices)
    for i in completed.into_iter().rev() {
        pending.remove(i);
    }

    events
}

pub fn start_dig(grid: &Grid, pos: GridPos, pending: &mut Vec<DigState>) -> bool {
    let cell = match grid.get(pos) {
        Some(c) => c,
        None => return false,
    };

    if let Some(ticks) = cell.material.dig_ticks() {
        // Don't duplicate dig requests
        if !pending.iter().any(|d| d.target == pos) {
            pending.push(DigState { target: pos, ticks_remaining: ticks });
        }
        true
    } else {
        false
    }
}

pub fn update_stability(grid: &mut Grid) -> Vec<TerrainEvent> {
    let mut events = Vec::new();
    let width = grid.width;
    let height = grid.height;

    // Process cells from bottom to top (y descending) for proper chain collapse
    for y in (0..height).rev() {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            let material = match grid.get(pos) {
                Some(c) => c.material,
                None => continue,
            };

            if !material.is_terrain() {
                continue;
            }

            let has_support = match grid.cell_below(pos) {
                Some(below) => below.material.is_solid() && !below.material.is_terrain(),
                None => false, // bottom row: no support below grid
            };

            let cell = grid.get_mut(pos).unwrap();

            if has_support || y == height - 1 {
                // Supported or on bedrock row: recover
                cell.stability = cell.stability.saturating_add(STABILITY_RECOVERY).min(255);
            } else if cell.stability > 0 {
                // No solid support underneath: decay
                let before = cell.stability;
                cell.stability = cell.stability.saturating_sub(STABILITY_DECAY);

                if cell.stability == 0 && before > 0 {
                    let from = cell.material;
                    cell.material = Material::LooseDirt;
                    events.push(TerrainEvent::Collapse { pos, from, to: Material::LooseDirt });
                }
            }

            // If already LooseDirt with 0 stability and unsupported, turn to Air
            if cell.material == Material::LooseDirt && cell.stability == 0 {
                if let Some(below) = grid.cell_below(pos) {
                    if !below.material.is_solid() || below.material.is_terrain() {
                        cell.material = Material::Air;
                        events.push(TerrainEvent::Collapse { pos, from: Material::LooseDirt, to: Material::Air });
                    }
                }
            }
        }
    }

    // Check for chain collapses
    if !events.is_empty() {
        let positions: Vec<GridPos> = events.iter().map(|e| match e {
            TerrainEvent::Collapse { pos, .. } => *pos,
            _ => unreachable!(),
        }).collect();
        if positions.len() > 1 {
            events.push(TerrainEvent::CollapseChain { positions });
        }
    }

    events
}
```

- [ ] **Step 2: Write terrain tests in same file**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Grid, Material, GridPos};

    #[test]
    fn test_start_dig_dirt() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Dirt);
        let mut pending = Vec::new();

        assert!(start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].target, GridPos::new(5, 5));
        assert_eq!(pending[0].ticks_remaining, 1);
    }

    #[test]
    fn test_start_dig_stone_fails() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Stone);
        let mut pending = Vec::new();

        assert!(!start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert!(pending.is_empty());
    }

    #[test]
    fn test_start_dig_no_duplicate() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Dirt);
        let mut pending = Vec::new();

        assert!(start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert!(!start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_process_digging_single_step() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Dirt);
        let mut pending = vec![DigState { target: GridPos::new(5, 5), ticks_remaining: 1 }];

        let events = process_digging(&mut grid, &mut pending);

        assert!(pending.is_empty());
        assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().material, Material::Air);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TerrainEvent::DigComplete { .. }));
    }

    #[test]
    fn test_process_digging_wet_dirt_two_step() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::WetDirt);
        let mut pending = vec![DigState { target: GridPos::new(5, 5), ticks_remaining: 2 }];

        // First tick
        let events = process_digging(&mut grid, &mut pending);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].ticks_remaining, 1);
        // Material unchanged on first tick for WetDirt->LooseDirt
        // (ticks_remaining > 0, so only decrement)

        // Second tick
        let events2 = process_digging(&mut grid, &mut pending);
        assert_eq!(pending.len(), 1); // still pending: LooseDirt -> Air needs another step? No... 
        // Actually WetDirt 2 ticks -> becomes LooseDirt on last tick
        // Let's check: after ticks_remaining hits 0, it becomes LooseDirt
        assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().material, Material::LooseDirt);

        // LooseDirt needs 0 ticks -> immediate Air
        start_dig(&grid, GridPos::new(5, 5), &mut pending);
        let events3 = process_digging(&mut grid, &mut pending);
        assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().material, Material::Air);
    }

    #[test]
    fn test_unsupported_dirt_collapses() {
        let mut grid = Grid::new(5, 10);
        // Place a dirt cell with air below it
        grid.set_material(GridPos::new(2, 5), Material::Dirt);
        // Below is Air (default)
        assert_eq!(grid.get(GridPos::new(2, 6)).unwrap().material, Material::Air);

        // Tick stability many times
        for _ in 0..40 {
            update_stability(&mut grid);
        }

        // Should have collapsed
        assert_eq!(grid.get(GridPos::new(2, 5)).unwrap().material, Material::Air);
    }

    #[test]
    fn test_supported_dirt_stays() {
        let mut grid = Grid::new(5, 10);
        // Place dirt on top of stone (stone is solid and not terrain)
        grid.set_material(GridPos::new(2, 6), Material::Stone);
        grid.set_material(GridPos::new(2, 5), Material::Dirt);

        for _ in 0..100 {
            update_stability(&mut grid);
        }

        // Dirt should still be there, supported by stone
        assert_eq!(grid.get(GridPos::new(2, 5)).unwrap().material, Material::Dirt);
        // Stability should have recovered to max
        assert_eq!(grid.get(GridPos::new(2, 5)).unwrap().stability, 255);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_simulation/src/terrain.rs
git commit -m "feat: add terrain physics — digging and structural stability"
```

---

### Task 5: Define tick engine

**Files:**
- Create: `crates/ant_simulation/src/tick.rs`

- [ ] **Step 1: Write tick.rs**

```rust
use crate::grid::Grid;
use crate::terrain::{DigState, TerrainEvent, process_digging, update_stability};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Paused,
    Normal,
    Fast,
    Fastest,
}

impl Speed {
    pub fn tick_interval_ms(self) -> Option<u64> {
        match self {
            Speed::Paused => None,
            Speed::Normal => Some(500),
            Speed::Fast => Some(250),
            Speed::Fastest => Some(100),
        }
    }

    pub fn ticks_per_second(self) -> u8 {
        match self {
            Speed::Paused => 0,
            Speed::Normal => 2,
            Speed::Fast => 4,
            Speed::Fastest => 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TickResult {
    pub tick: u64,
    pub events: Vec<TerrainEvent>,
}

pub struct Simulation {
    pub grid: Grid,
    pub tick: u64,
    pub speed: Speed,
    pub events: Vec<TerrainEvent>,
    pub pending_digs: Vec<DigState>,
}

impl Simulation {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: Grid::new(width, height),
            tick: 0,
            speed: Speed::Normal,
            events: Vec::new(),
            pending_digs: Vec::new(),
        }
    }

    pub fn tick(&mut self) -> TickResult {
        self.tick += 1;
        self.events.clear();

        let dig_events = process_digging(&mut self.grid, &mut self.pending_digs);
        let stability_events = update_stability(&mut self.grid);

        self.events.extend(dig_events);
        self.events.extend(stability_events);

        TickResult {
            tick: self.tick,
            events: self.events.clone(),
        }
    }

    pub fn day(&self) -> u64 {
        self.tick / 2400
    }

    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
    }

    pub fn tick_interval_ms(&self) -> Option<u64> {
        self.speed.tick_interval_ms()
    }
}
```

- [ ] **Step 2: Write tick tests in same file**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_tick_interval() {
        assert_eq!(Speed::Paused.tick_interval_ms(), None);
        assert_eq!(Speed::Normal.tick_interval_ms(), Some(500));
        assert_eq!(Speed::Fast.tick_interval_ms(), Some(250));
        assert_eq!(Speed::Fastest.tick_interval_ms(), Some(100));
    }

    #[test]
    fn test_speed_ticks_per_second() {
        assert_eq!(Speed::Paused.ticks_per_second(), 0);
        assert_eq!(Speed::Normal.ticks_per_second(), 2);
        assert_eq!(Speed::Fast.ticks_per_second(), 4);
        assert_eq!(Speed::Fastest.ticks_per_second(), 10);
    }

    #[test]
    fn test_simulation_new() {
        let sim = Simulation::new(10, 10);
        assert_eq!(sim.tick, 0);
        assert_eq!(sim.speed, Speed::Normal);
        assert!(sim.pending_digs.is_empty());
    }

    #[test]
    fn test_simulation_tick_increments() {
        let mut sim = Simulation::new(10, 10);
        let result = sim.tick();
        assert_eq!(result.tick, 1);
        assert_eq!(sim.tick, 1);
    }

    #[test]
    fn test_simulation_day() {
        let mut sim = Simulation::new(10, 10);
        assert_eq!(sim.day(), 0);
        // Manually set tick
        sim.tick = 2400;
        assert_eq!(sim.day(), 1);
        sim.tick = 4800;
        assert_eq!(sim.day(), 2);
    }

    #[test]
    fn test_set_speed() {
        let mut sim = Simulation::new(10, 10);
        sim.set_speed(Speed::Fast);
        assert_eq!(sim.speed, Speed::Fast);
        assert_eq!(sim.tick_interval_ms(), Some(250));
    }

    #[test]
    fn test_paused_no_ticks() {
        let mut sim = Simulation::new(10, 10);
        sim.set_speed(Speed::Paused);
        assert_eq!(sim.speed.ticks_per_second(), 0);
        assert_eq!(sim.tick_interval_ms(), None);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_simulation/src/tick.rs
git commit -m "feat: add tick engine with Speed enum and Simulation struct"
```

---

### Task 6: Define Snapshot and Command interface

**Files:**
- Create: `crates/ant_simulation/src/snapshot.rs`

- [ ] **Step 1: Write snapshot.rs**

```rust
use crate::grid::{Grid, GridPos, Material};
use crate::terrain::TerrainEvent;
use crate::tick::Speed;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<CellSnapshot>,
    pub events: Vec<TerrainEvent>,
}

#[derive(Debug, Clone, Copy)]
pub struct CellSnapshot {
    pub material: Material,
    pub stability: u8,
}

#[derive(Debug, Clone)]
pub enum Command {
    AddFood { x: u16, y: u16 },
    AddWater { x: u16, y: u16 },
    ModifyTerrain { x: u16, y: u16, material: Material },
    SetSpeed(Speed),
}

impl Snapshot {
    pub fn from_simulation(sim: &crate::tick::Simulation) -> Self {
        let cells: Vec<CellSnapshot> = sim.grid.cells.iter().map(|c| CellSnapshot {
            material: c.material,
            stability: c.stability,
        }).collect();

        Self {
            tick: sim.tick,
            width: sim.grid.width,
            height: sim.grid.height,
            cells,
            events: sim.events.clone(),
        }
    }
}
```

- [ ] **Step 2: Write tests in same file**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick::Simulation;
    use crate::grid::Material;

    #[test]
    fn test_snapshot_from_simulation() {
        let mut sim = Simulation::new(5, 5);
        sim.tick(); // advance once
        let snap = Snapshot::from_simulation(&sim);

        assert_eq!(snap.tick, 1);
        assert_eq!(snap.width, 5);
        assert_eq!(snap.height, 5);
        assert_eq!(snap.cells.len(), 25);
        assert!(snap.cells.iter().all(|c| c.material == Material::Air));
    }

    #[test]
    fn test_cell_snapshot_matches_grid() {
        let mut sim = Simulation::new(5, 5);
        sim.grid.set_material(GridPos::new(2, 2), Material::Dirt);
        let snap = Snapshot::from_simulation(&sim);

        let dirt_cell = snap.cells[2 * 5 + 2];
        assert_eq!(dirt_cell.material, Material::Dirt);
        assert_eq!(dirt_cell.stability, 128);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_simulation/src/snapshot.rs
git commit -m "feat: add Snapshot and Command types for sim-renderer interface"
```

---

### Task 7: Persistence — save and load with bincode

**Files:**
- Create: `crates/ant_simulation/src/persistence.rs`

- [ ] **Step 1: Write persistence.rs**

```rust
use serde::{Serialize, Deserialize};
use crate::grid::{Grid, Cell, Material, PheromoneLayer};
use crate::tick::Simulation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<SavedCell>,
    pub rng_state: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCell {
    pub material: Material,
    pub humidity: u8,
    pub temperature: i16,
    pub stability: u8,
    pub pheromones: PheromoneLayer,
    pub organic_matter: u8,
}

impl SaveFile {
    pub fn from_simulation(sim: &Simulation) -> Self {
        let cells: Vec<SavedCell> = sim.grid.cells.iter().map(|c| SavedCell {
            material: c.material,
            humidity: c.humidity,
            temperature: c.temperature,
            stability: c.stability,
            pheromones: c.pheromones,
            organic_matter: c.organic_matter,
        }).collect();

        Self {
            version: 1,
            tick: sim.tick,
            width: sim.grid.width,
            height: sim.grid.height,
            cells,
            rng_state: [0u8; 32],
        }
    }

    pub fn to_simulation(&self) -> Simulation {
        let mut sim = Simulation::new(self.width, self.height);
        sim.tick = self.tick;
        for (i, saved) in self.cells.iter().enumerate() {
            sim.grid.cells[i] = Cell {
                material: saved.material,
                humidity: saved.humidity,
                temperature: saved.temperature,
                stability: saved.stability,
                pheromones: saved.pheromones,
                organic_matter: saved.organic_matter,
            };
        }
        sim
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::error::DecodeError> {
        bincode::serde::decode_from_slice(data, bincode::config::standard()).map(|(val, _)| val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Material;

    #[test]
    fn test_save_load_roundtrip() {
        let mut sim = Simulation::new(10, 10);
        sim.tick = 42;
        sim.grid.set_material(crate::grid::GridPos::new(3, 3), Material::Dirt);

        let save = SaveFile::from_simulation(&sim);
        let bytes = save.to_bytes().unwrap();
        let loaded = SaveFile::from_bytes(&bytes).unwrap();
        let restored = loaded.to_simulation();

        assert_eq!(restored.tick, 42);
        assert_eq!(restored.grid.width, 10);
        assert_eq!(restored.grid.height, 10);
        assert_eq!(restored.grid.get(crate::grid::GridPos::new(3, 3)).unwrap().material, Material::Dirt);
    }

    #[test]
    fn test_save_version_is_1() {
        let sim = Simulation::new(5, 5);
        let save = SaveFile::from_simulation(&sim);
        assert_eq!(save.version, 1);
    }

    #[test]
    fn test_invalid_bytes_returns_err() {
        let result = SaveFile::from_bytes(&[0xFF, 0xFF, 0xFF]);
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All tests pass, including roundtrip test.

- [ ] **Step 3: Commit**

```bash
git add crates/ant_simulation/src/persistence.rs
git commit -m "feat: add persistence with bincode save/load and roundtrip"
```

---

### Task 8: World generation — initial terrarium state

**Files:**
- Modify: `crates/ant_simulation/src/grid.rs` (add `generate_initial_world`)

- [ ] **Step 1: Add world generation function to grid.rs**

```rust
impl Grid {
    // ... existing methods ...

    pub fn generate_initial_world(width: u16, height: u16) -> Self {
        let mut grid = Self::new(width, height);
        let surface_y = height / 4; // top 25% air, rest terrain

        // Layer: Air from top to surface_y
        // (already Air by default)

        // Surface layer: thin grass/dirt mix (y = surface_y)
        for x in 0..width {
            grid.set_material(GridPos::new(x, surface_y), Material::Dirt);
        }

        // Dirt strata: surface_y+1 down to ~80% of height
        let stone_y = height - (height / 8); // bottom 12.5% is stone
        for y in (surface_y + 1)..stone_y {
            for x in 0..width {
                let material = if y % 6 == 0 && x % 4 == 0 {
                    // Occasional variation
                    if y % 12 == 0 {
                        Material::Sand
                    } else {
                        Material::Dirt
                    }
                } else {
                    Material::Dirt
                };
                grid.set_material(GridPos::new(x, y), material);
            }
        }

        // Stone bedrock: bottom layer
        for y in stone_y..height {
            for x in 0..width {
                grid.set_material(GridPos::new(x, y), Material::Stone);
            }
        }

        grid
    }

    pub fn queen_position(&self) -> GridPos {
        let y = self.surface_y();
        GridPos::new(self.width / 2, y)
    }
}
```

- [ ] **Step 2: Write test**

```rust
#[test]
fn test_generate_initial_world() {
    let grid = Grid::generate_initial_world(128, 96);
    assert_eq!(grid.width, 128);
    assert_eq!(grid.height, 96);

    // Top should be air
    assert_eq!(grid.get(GridPos::new(64, 0)).unwrap().material, Material::Air);

    // Surface should be dirt
    let sy = grid.surface_y();
    assert_eq!(grid.get(GridPos::new(64, sy)).unwrap().material, Material::Dirt);

    // Bottom should be stone
    assert_eq!(grid.get(GridPos::new(64, 95)).unwrap().material, Material::Stone);

    // Queen position should be at surface, centered
    let qpos = grid.queen_position();
    assert_eq!(qpos.x, 64);
    assert_eq!(qpos.y, sy);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_simulation/src/grid.rs
git commit -m "feat: add initial world generation with layered terrain"
```

---

## Phase 2: Renderer Crate

### Task 9: Create ant_renderer crate scaffold with Bevy plugin

**Files:**
- Modify: `crates/ant_renderer/src/lib.rs`
- Create: `crates/ant_renderer/src/app.rs`
- Create: `crates/ant_renderer/src/assets.rs`

- [ ] **Step 1: Rewrite lib.rs to export the plugin**

```rust
pub mod app;
pub mod camera;
pub mod sprites;
pub mod hud;
pub mod input;
pub mod assets;

pub use app::TerrariumPlugin;

pub fn run() {
    use bevy::prelude::*;
    App::new()
        .add_plugins((
            bevy::window::WindowPlugin::default(),
            bevy::sprite::SpritePlugin::default(),
            bevy::ui::UiPlugin::default(),
            bevy::input::InputPlugin::default(),
        ))
        .add_plugins(TerrariumPlugin)
        .run();
}
```

- [ ] **Step 2: Write assets.rs (palette + constants)**

```rust
use bevy::prelude::*;

pub const CELL_SIZE: f32 = 8.0;
pub const GRID_WIDTH: u16 = 128;
pub const GRID_HEIGHT: u16 = 96;
pub const QUEEN_COLOR: Color = Color::srgb(0.91, 0.75, 0.25); // #E8C040

pub fn material_color(material: ant_simulation::grid::Material) -> Color {
    match material {
        ant_simulation::grid::Material::Air => Color::srgb(0.227, 0.290, 0.416),     // #3A4A6A
        ant_simulation::grid::Material::Dirt => Color::srgb(0.545, 0.412, 0.078),     // #8B6914
        ant_simulation::grid::Material::LooseDirt => Color::srgb(0.678, 0.545, 0.235),// #AD8B3C
        ant_simulation::grid::Material::WetDirt => Color::srgb(0.38, 0.29, 0.16),     // darker brown
        ant_simulation::grid::Material::Sand => Color::srgb(0.761, 0.698, 0.502),     // #C2B280
        ant_simulation::grid::Material::Stone => Color::srgb(0.408, 0.408, 0.408),    // #686868
        ant_simulation::grid::Material::Water => Color::srgb(0.165, 0.353, 0.541),    // #2A5A8A
        ant_simulation::grid::Material::Food => Color::srgb(0.91, 0.75, 0.25),        // #E8C040
        ant_simulation::grid::Material::OrganicWaste => Color::srgb(0.4, 0.3, 0.2),
        ant_simulation::grid::Material::Fungus => Color::srgb(0.3, 0.6, 0.3),
        ant_simulation::grid::Material::Egg => Color::srgb(0.95, 0.95, 0.85),
        ant_simulation::grid::Material::Larva => Color::srgb(1.0, 0.9, 0.7),
    }
}
```

- [ ] **Step 3: Write app.rs (plugin setup)**

```rust
use bevy::prelude::*;
use ant_simulation::tick::{Simulation, Speed};
use crate::{
    sprites::{self, SimulationState},
    input,
    hud::{self, HudState},
    camera,
};

pub struct TerrariumPlugin;

impl Plugin for TerrariumPlugin {
    fn build(&self, app: &mut App) {
        let simulation = Simulation::from_grid(
            ant_simulation::grid::Grid::generate_initial_world(
                crate::assets::GRID_WIDTH,
                crate::assets::GRID_HEIGHT,
            )
        );

        app.insert_resource(simulation);
        app.insert_resource(SimulationState::default());
        app.insert_resource(HudState::default());
        app.insert_resource(input::InputState::default());

        app.add_systems(Startup, (
            camera::setup_camera,
            sprites::setup_grid_sprites,
            hud::setup_hud,
        ));

        app.add_systems(Update, (
            input::handle_input,
            crate::sprites::tick_simulation,
            crate::sprites::apply_snapshot,
            hud::update_hud,
        ));
    }
}
```

- [ ] **Step 4: Create placeholder module files**

Create empty placeholder files:
- `crates/ant_renderer/src/camera.rs` with `// Camera system`
- `crates/ant_renderer/src/sprites.rs` with `// Grid sprites`
- `crates/ant_renderer/src/hud.rs` with `// HUD`
- `crates/ant_renderer/src/input.rs` with `// Input handling`

- [ ] **Step 5: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully. There will be dead_code warnings but no errors.

- [ ] **Step 6: Commit**

```bash
git add crates/ant_renderer/
git commit -m "feat: scaffold ant_renderer crate with Bevy plugin structure"
```

---

### Task 10: Implement sprites — grid rendering

**Files:**
- Modify: `crates/ant_renderer/src/sprites.rs`
- Modify: `crates/ant_renderer/src/app.rs` (update to use correct Simulation constructor, Simulation::from_grid requires it)

- [ ] **Step 1: Note — add from_grid constructor to Simulation**

First, add this constructor to `crates/ant_simulation/src/tick.rs`:

```rust
impl Simulation {
    // ... existing ...

    pub fn from_grid(grid: Grid) -> Self {
        Self {
            grid,
            tick: 0,
            speed: Speed::Normal,
            events: Vec::new(),
            pending_digs: Vec::new(),
        }
    }
}
```

Add tests:
```rust
#[test]
fn test_simulation_from_grid() {
    let grid = Grid::generate_initial_world(10, 10);
    let sim = Simulation::from_grid(grid);
    assert_eq!(sim.tick, 0);
    assert_eq!(sim.grid.width, 10);
}
```

- [ ] **Step 2: Write sprites.rs**

```rust
use bevy::prelude::*;
use ant_simulation::{
    snapshot::Snapshot,
    tick::{Simulation, Speed},
};
use crate::assets::{self, CELL_SIZE, QUEEN_COLOR};

#[derive(Resource)]
pub struct SimulationState {
    pub snapshot: Option<Snapshot>,
    pub tick_timer: Timer,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            snapshot: None,
            tick_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

pub fn setup_grid_sprites(
    mut commands: Commands,
    simulation: Res<Simulation>,
) {
    let snap = Snapshot::from_simulation(&simulation);

    for y in 0..snap.height {
        for x in 0..snap.width {
            let idx = y as usize * snap.width as usize + x as usize;
            let cell = snap.cells[idx];
            let color = assets::material_color(cell.material);
            let world_x = x as f32 * CELL_SIZE;
            let world_y = -(y as f32 * CELL_SIZE); // y-up is negative for bevy 2D

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 0.0),
                CellSprite { grid_x: x as u16, grid_y: y as u16 },
            ));
        }
    }

    // Spawn queen marker
    let queen = simulation.grid.queen_position();
    commands.spawn((
        Sprite {
            color: QUEEN_COLOR,
            custom_size: Some(Vec2::splat(CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            queen.x as f32 * CELL_SIZE,
            -(queen.y as f32 * CELL_SIZE),
            1.0, // above grid
        ),
        QueenMarker,
    ));
}

#[derive(Component)]
pub struct CellSprite {
    pub grid_x: u16,
    pub grid_y: u16,
}

#[derive(Component)]
pub struct QueenMarker;

pub fn tick_simulation(
    time: Res<Time>,
    mut simulation: ResMut<Simulation>,
    mut state: ResMut<SimulationState>,
) {
    let interval_ms = match simulation.tick_interval_ms() {
        Some(ms) => ms as f32 / 1000.0,
        None => return,
    };

    state.tick_timer.tick(time.delta());
    if state.tick_timer.just_finished() {
        simulation.tick();
        state.snapshot = Some(Snapshot::from_simulation(&simulation));
    }
}

pub fn apply_snapshot(
    mut query: Query<(&CellSprite, &mut Sprite)>,
    simulation_state: Res<SimulationState>,
) {
    let snapshot = match &simulation_state.snapshot {
        Some(s) => s,
        None => return,
    };

    for (cell_sprite, mut sprite) in query.iter_mut() {
        let idx = cell_sprite.grid_y as usize * snapshot.width as usize + cell_sprite.grid_x as usize;
        if let Some(cell) = snapshot.cells.get(idx) {
            let material = cell.material;
            let mut color = assets::material_color(material);

            // Fade unstable cells
            if cell.stability < 64 {
                let alpha = cell.stability as f32 / 64.0;
                color.set_alpha(alpha.max(0.2));
            }

            sprite.color = color;
        }
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p ant_simulation`
Expected: All tests pass.

Run: `cargo check`
Expected: Compiles with all renderer modules.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_simulation/src/tick.rs crates/ant_renderer/src/sprites.rs crates/ant_renderer/src/app.rs
git commit -m "feat: implement grid sprite rendering with snapshot application"
```

---

### Task 11: Camera system — zoom, pan, snap

**Files:**
- Modify: `crates/ant_renderer/src/camera.rs`

- [ ] **Step 1: Write camera.rs**

```rust
use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use crate::assets::{GRID_WIDTH, GRID_HEIGHT, CELL_SIZE};

pub fn setup_camera(mut commands: Commands) {
    let grid_center_x = GRID_WIDTH as f32 * CELL_SIZE / 2.0;
    let grid_center_y = -(GRID_HEIGHT as f32 * CELL_SIZE / 2.0);

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.02, 0.06)),
            ..default()
        },
        Transform::from_xyz(grid_center_x, grid_center_y, 100.0),
        CameraState {
            min_scale: 0.5,
            max_scale: 4.0,
            scale: 1.0,
        },
    ));
}

#[derive(Component)]
pub struct CameraState {
    pub min_scale: f32,
    pub max_scale: f32,
    pub scale: f32,
}

pub fn zoom_camera(
    mut query: Query<(&mut OrthographicProjection, &mut CameraState)>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    for event in scroll_events.read() {
        for (mut proj, mut state) in query.iter_mut() {
            let zoom_factor = 1.0 - event.y * 0.1;
            state.scale = (state.scale * zoom_factor).clamp(state.min_scale, state.max_scale);
            proj.scale = 1.0 / state.scale;
        }
    }
}

pub fn pan_camera(
    mut query: Query<&mut Transform, With<Camera2d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    state: Query<&CameraState>,
) {
    if !mouse.pressed(MouseButton::Right) {
        return;
    }

    let scale = state.single().scale;
    for event in motion.read() {
        for mut transform in query.iter_mut() {
            transform.translation.x -= event.delta.x * scale;
            transform.translation.y += event.delta.y * scale;
        }
    }
}

pub fn snap_to_queen(
    keyboard: Res<ButtonInput<KeyCode>>,
    simulation: Res<ant_simulation::tick::Simulation>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        let queen = simulation.grid.queen_position();
        for mut transform in query.iter_mut() {
            transform.translation.x = queen.x as f32 * CELL_SIZE;
            transform.translation.y = -(queen.y as f32 * CELL_SIZE);
        }
    }
}

pub fn keyboard_pan(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    state: Query<&CameraState>,
) {
    let scale = state.single().scale;
    let speed = 4.0 * scale;
    for mut transform in query.iter_mut() {
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            transform.translation.y += speed;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= speed;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= speed;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            transform.translation.x += speed;
        }
    }
}
```

- [ ] **Step 2: Wire camera systems into app.rs**

Update `app.rs` to add camera systems:

```rust
// In TerrariumPlugin::build(), add these system registrations:
app.add_systems(Update, (
    camera::zoom_camera,
    camera::pan_camera,
    camera::keyboard_pan,
    camera::snap_to_queen,
    // ... existing systems
));
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_renderer/src/camera.rs crates/ant_renderer/src/app.rs
git commit -m "feat: add camera system with zoom, pan, keyboard pan, and queen snap"
```

---

### Task 12: Input handling — mouse clicks to commands

**Files:**
- Modify: `crates/ant_renderer/src/input.rs`
- Modify: `crates/ant_renderer/src/app.rs`

- [ ] **Step 1: Write input.rs**

```rust
use bevy::prelude::*;
use ant_simulation::{
    grid::Material,
    snapshot::Command,
    tick::{Simulation, Speed},
};
use crate::assets::{CELL_SIZE, GRID_WIDTH, GRID_HEIGHT};

#[derive(Resource, Default)]
pub struct InputState {
    pub pending_commands: Vec<Command>,
}

pub fn handle_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut simulation: ResMut<Simulation>,
    mut state: ResMut<InputState>,
) {
    state.pending_commands.clear();

    // Speed controls
    if keyboard.just_pressed(KeyCode::Space) {
        let new_speed = match simulation.speed {
            Speed::Paused => Speed::Normal,
            _ => Speed::Paused,
        };
        state.pending_commands.push(Command::SetSpeed(new_speed));
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        state.pending_commands.push(Command::SetSpeed(Speed::Normal));
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        state.pending_commands.push(Command::SetSpeed(Speed::Fast));
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        state.pending_commands.push(Command::SetSpeed(Speed::Fastest));
    }

    // Mouse world position
    let Ok(window) = window.get_single() else { return };
    let cursor = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let Ok((camera, cam_transform)) = camera.get_single() else { return };

    let world_pos = match camera.viewport_to_world_2d(cam_transform, cursor) {
        Some(pos) => pos,
        None => return,
    };

    let grid_x = (world_pos.x / CELL_SIZE).floor();
    let grid_y = (-world_pos.y / CELL_SIZE).floor();

    if grid_x < 0.0 || grid_y < 0.0 || grid_x >= GRID_WIDTH as f32 || grid_y >= GRID_HEIGHT as f32 {
        return;
    }

    let x = grid_x as u16;
    let y = grid_y as u16;

    // Left click: deposit food (only on Air cells at surface)
    if mouse.just_pressed(MouseButton::Left) && !keyboard.pressed(KeyCode::ShiftLeft) {
        let cell = simulation.grid.get(ant_simulation::grid::GridPos::new(x, y));
        if let Some(cell) = cell {
            if cell.material == Material::Air && y == simulation.grid.surface_y() {
                state.pending_commands.push(Command::AddFood { x, y });
            }
        }
    }

    // Right click: add water (on Dirt/Sand cells)
    if mouse.just_pressed(MouseButton::Right) {
        let cell = simulation.grid.get(ant_simulation::grid::GridPos::new(x, y));
        if let Some(cell) = cell {
            if matches!(cell.material, Material::Dirt | Material::Sand) {
                state.pending_commands.push(Command::AddWater { x, y });
            }
        }
    }

    // Shift + left click: modify terrain (top 3 rows, toggle Dirt/Air)
    if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft) {
        if y <= 3 {
            let cell = simulation.grid.get(ant_simulation::grid::GridPos::new(x, y));
            if let Some(cell) = cell {
                let new_material = if cell.material == Material::Air { Material::Dirt } else { Material::Air };
                state.pending_commands.push(Command::ModifyTerrain { x, y, material: new_material });
            }
        }
    }

    // Ctrl+S: save
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyS) {
        // Save will be handled in a future step
        info!("Save requested (not yet wired)");
    }

    // Apply commands to simulation
    for cmd in &state.pending_commands {
        match cmd {
            Command::AddFood { x, y } => {
                simulation.grid.set_material(
                    ant_simulation::grid::GridPos::new(*x, *y),
                    Material::Food,
                );
            }
            Command::AddWater { x, y } => {
                simulation.grid.set_material(
                    ant_simulation::grid::GridPos::new(*x, *y),
                    Material::WetDirt,
                );
            }
            Command::ModifyTerrain { x, y, material } => {
                simulation.grid.set_material(
                    ant_simulation::grid::GridPos::new(*x, *y),
                    *material,
                );
            }
            Command::SetSpeed(speed) => {
                simulation.set_speed(*speed);
            }
        }
    }
}
```

- [ ] **Step 2: Wire input system into app.rs**

Add to app.rs Update systems:
```rust
app.add_systems(Update, (
    input::handle_input,
    // ... existing
));
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_renderer/src/input.rs crates/ant_renderer/src/app.rs
git commit -m "feat: add input handling for mouse clicks, keyboard speed controls"
```

---

### Task 13: HUD — bottom bar and side panel

**Files:**
- Modify: `crates/ant_renderer/src/hud.rs`

- [ ] **Step 1: Write hud.rs**

```rust
use bevy::prelude::*;
use ant_simulation::tick::{Simulation, Speed};

#[derive(Resource)]
pub struct HudState {
    pub panel_visible: bool,
}

impl Default for HudState {
    fn default() -> Self {
        Self { panel_visible: false }
    }
}

#[derive(Component)]
pub struct BottomBar;

#[derive(Component)]
pub struct TickText;

#[derive(Component)]
pub struct SpeedText;

#[derive(Component)]
pub struct QueenStatusText;

#[derive(Component)]
pub struct WorkerCountText;

#[derive(Component)]
pub struct SidePanel;

pub fn setup_hud(mut commands: Commands) {
    // Bottom bar
    commands.spawn((
        BottomBar,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(32.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        // Speed buttons
        parent.spawn((
            Button,
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        )).with_children(|btn| {
            btn.spawn(Text::new("||"));
        });

        parent.spawn((
            Button,
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        )).with_children(|btn| {
            btn.spawn(Text::new(">"));
        });

        parent.spawn((
            Button,
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        )).with_children(|btn| {
            btn.spawn(Text::new(">>"));
        });

        // Separator
        parent.spawn(Node {
            width: Val::Px(2.0),
            height: Val::Px(20.0),
            ..default()
        }).insert(BackgroundColor(Color::srgb(0.3, 0.3, 0.3)));

        // Tick counter
        parent.spawn((
            TickText,
            Text::new("Tick: 0"),
            Node { margin: UiRect::horizontal(Val::Px(12.0)), ..default() },
        ));

        // Speed label
        parent.spawn((
            SpeedText,
            Text::new("1x"),
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        ));

        // Queen status
        parent.spawn((
            QueenStatusText,
            Text::new("Queen: ●"),
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        ));

        // Worker count
        parent.spawn((
            WorkerCountText,
            Text::new("Workers: 0"),
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        ));
    });

    // Side panel (hidden by default)
    commands.spawn((
        SidePanel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            top: Val::Px(40.0),
            width: Val::Px(220.0),
            height: Val::Px(400.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
    )).with_children(|parent| {
        parent.spawn(Text::new("Queen: ● (decorative only)\n\nEggs: 0\nLarvae: 0\nWorkers: 0\n\nAvg Humidity: 30%\nTemperature: 22.0°C\nTunnels dug: 0\nCollapses: 0"));
    });
}

pub fn update_hud(
    simulation: Res<Simulation>,
    mut tick_text: Query<&mut Text, (With<TickText>, Without<SpeedText>)>,
    mut speed_text: Query<&mut Text, (With<SpeedText>, Without<TickText>)>,
    mut queen_text: Query<&mut Text, (With<QueenStatusText>, Without<TickText>, Without<SpeedText>)>,
    mut worker_text: Query<&mut Text, (With<WorkerCountText>, Without<TickText>, Without<SpeedText>, Without<QueenStatusText>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel: Query<&mut Node, With<SidePanel>>,
    mut hud_state: ResMut<HudState>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        hud_state.panel_visible = !hud_state.panel_visible;
        for mut node in panel.iter_mut() {
            node.display = if hud_state.panel_visible { Display::Flex } else { Display::None };
        }
    }

    for mut text in tick_text.iter_mut() {
        text.0 = format!("Tick: {}  |  Day: {}", simulation.tick, simulation.day());
    }

    for mut text in speed_text.iter_mut() {
        let label = match simulation.speed {
            Speed::Paused => "||",
            Speed::Normal => "1x",
            Speed::Fast => "2x",
            Speed::Fastest => "4x",
        };
        text.0 = label.to_string();
    }

    for mut text in queen_text.iter_mut() {
        text.0 = "Queen: ●".to_string();
    }

    for mut text in worker_text.iter_mut() {
        text.0 = format!("Workers: 0  |  Food: 0");
    }
}
```

- [ ] **Step 2: Wire HUD into app.rs**

Add to Update systems:
```rust
app.add_systems(Update, (
    // ... existing
    hud::update_hud,
));
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_renderer/src/hud.rs crates/ant_renderer/src/app.rs
git commit -m "feat: add HUD with bottom bar and toggleable side panel"
```

---

### Task 14: Glass terrarium overlay

**Files:**
- Modify: `crates/ant_renderer/src/sprites.rs` (add overlay setup)
- Modify: `crates/ant_renderer/src/app.rs` (add overlay system to Startup)

- [ ] **Step 1: Add glass overlay entity to sprites.rs**

```rust
#[derive(Component)]
pub struct GlassOverlay;

pub fn setup_glass_overlay(mut commands: Commands) {
    let width_px = GRID_WIDTH as f32 * CELL_SIZE;
    let height_px = GRID_HEIGHT as f32 * CELL_SIZE;
    let center_x = width_px / 2.0;
    let center_y = -height_px / 2.0;

    // Glass border: 4 thin lines forming a frame
    let border_color = Color::srgba(0.23, 0.23, 0.35, 1.0); // #3A3A5A
    let border_thickness = 3.0;

    // Top border
    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(width_px, border_thickness)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y - height_px / 2.0, 5.0),
    ));

    // Bottom border
    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(width_px, border_thickness)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y + height_px / 2.0, 5.0),
    ));

    // Left border
    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(border_thickness, height_px)),
            ..default()
        },
        Transform::from_xyz(center_x - width_px / 2.0, center_y, 5.0),
    ));

    // Right border
    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(border_thickness, height_px)),
            ..default()
        },
        Transform::from_xyz(center_x + width_px / 2.0, center_y, 5.0),
    ));

    // Subtle reflection gradient at top 30%
    commands.spawn((
        GlassOverlay,
        Sprite {
            color: Color::srgba(0.59, 0.78, 1.0, 0.08),
            custom_size: Some(Vec2::new(width_px, height_px * 0.30)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y - height_px * 0.35, 5.0),
    ));
}
```

- [ ] **Step 2: Wire into app.rs Startup**

Add `crate::sprites::setup_glass_overlay` to Startup systems.

- [ ] **Step 3: Verify compilation and run**

Run: `cargo check`
Expected: Compiles.

Run: `cargo run`
Expected: Window opens showing terrarium with glass frame border, grid of cells, and HUD. The yellow queen dot should be visible at the surface center.

- [ ] **Step 4: Commit**

```bash
git add crates/ant_renderer/src/sprites.rs crates/ant_renderer/src/app.rs
git commit -m "feat: add glass terrarium overlay with border and reflection"
```

---

### Task 15: Wire persistence into the app

**Files:**
- Modify: `crates/ant_renderer/src/input.rs` (add save on Ctrl+S, load on startup)
- Modify: `crates/ant_renderer/src/app.rs`

- [ ] **Step 1: Update input.rs— replace the save placeholder**

In `handle_input`, replace the "Save requested" info with:

```rust
// Ctrl+S: save
if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyS) {
    let save = ant_simulation::persistence::SaveFile::from_simulation(&simulation);
    if let Ok(bytes) = save.to_bytes() {
        let path = save_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &bytes);
        info!("Saved to {:?}", path);
    }
}

// Ctrl+L: load
if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyL) {
    let path = save_path();
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(save) = ant_simulation::persistence::SaveFile::from_bytes(&bytes) {
            *simulation = save.to_simulation();
            info!("Loaded from {:?}", path);
        }
    }
}
```

Add this function above `handle_input`:

```rust
fn save_path() -> std::path::PathBuf {
    let appdata = std::env::var("APPDATA")
        .unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(appdata)
        .join("ant_terrarium")
        .join("saves")
        .join("save_001.bin")
}
```

- [ ] **Step 2: Add auto-save system**

Add this system to `input.rs`:

```rust
pub fn auto_save(
    time: Res<Time>,
    simulation: Res<Simulation>,
    mut timer: Local<Timer>,
) {
    // Initialize timer on first run
    if timer.duration().as_secs_f32() == 0.0 {
        *timer = Timer::from_seconds(60.0, TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        let save = ant_simulation::persistence::SaveFile::from_simulation(&simulation);
        if let Ok(bytes) = save.to_bytes() {
            let path = save_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, &bytes);
            info!("Auto-saved tick {}", simulation.tick);
        }
    }
}
```

- [ ] **Step 3: Update app.rs — auto-load on startup + register auto-save**

In `TerrariumPlugin::build()`, load save on startup:

```rust
fn build(&self, app: &mut App) {
    // Try to load existing save, otherwise use fresh world
    let simulation = Self::load_or_create();

    app.insert_resource(simulation);
    // ... rest same as before
}

fn load_or_create() -> Simulation {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let path = std::path::PathBuf::from(appdata)
        .join("ant_terrarium")
        .join("saves")
        .join("save_001.bin");

    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(save) = ant_simulation::persistence::SaveFile::from_bytes(&bytes) {
            info!("Loaded save from {:?}", path);
            return save.to_simulation();
        }
    }

    info!("No save found, creating fresh world");
    Simulation::from_grid(
        ant_simulation::grid::Grid::generate_initial_world(
            GRID_WIDTH,
            GRID_HEIGHT,
        )
    )
}
```

Register auto-save in app.rs Update systems alongside other input systems:
```rust
app.add_systems(Update, (
    input::handle_input,
    input::auto_save,
    // ... other systems
));
```

- [ ] **Step 4: Verify**

Run: `cargo check`
Expected: Compiles.

Run: `cargo run`
Expected: Window opens. Press Ctrl+S, close, reopen — world should restore.

- [ ] **Step 5: Commit**

```bash
git add crates/ant_renderer/src/input.rs crates/ant_renderer/src/app.rs
git commit -m "feat: wire save/load persistence with Ctrl+S/Ctrl+L and auto-load"
```

---

### Task 16: Final integration — window config, cleanup, and full test

**Files:**
- Modify: `src/main.rs`
- Modify: `crates/ant_renderer/src/lib.rs`

- [ ] **Step 1: Update main.rs for window configuration**

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            bevy::window::WindowPlugin {
                primary_window: Some(Window {
                    title: "Ant Terrarium".into(),
                    resolution: (1024.0, 768.0).into(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            },
            bevy::sprite::SpritePlugin::default(),
            bevy::ui::UiPlugin::default(),
            bevy::input::InputPlugin::default(),
        ))
        .add_plugins(ant_renderer::TerrariumPlugin)
        .run();
}
```

- [ ] **Step 2: Update lib.rs — re-export needed types, remove duplicate app creation**

```rust
pub mod app;
pub mod camera;
pub mod sprites;
pub mod hud;
pub mod input;
pub mod assets;

pub use app::TerrariumPlugin;
```

Remove the `run()` function from `lib.rs` since `main.rs` now handles app creation directly.

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: All tests in ant_simulation pass.

Run: `cargo check`
Expected: No errors.

Run: `cargo build --release`
Expected: Clean release build.

- [ ] **Step 4: Final run verification**

Run: `cargo run`
Expected:
- Window opens at 1024x768 with title "Ant Terrarium"
- Terrarium visible with blue air at top, brown dirt, gray stone at bottom
- Yellow queen dot at center of surface
- Glass frame border around terrarium
- Bottom bar with speed controls, tick counter, queen/worker status
- Space pauses/resumes, 1/2/3 changes speed
- Scroll wheel zooms, right-drag pans, WASD pans
- Q snaps to queen
- Tab toggles side panel
- Clicking surface adds food (amber cell)
- Right-clicking dirt adds water (darker dirt)
- Shift-click top 3 rows toggles terrain
- Ctrl+S saves, Ctrl+L loads

- [ ] **Step 5: Commit**

```bash
git add src/main.rs crates/ant_renderer/src/lib.rs
git commit -m "feat: finalize window config, remove duplicate run(), integration complete"
```

---

## Test Summary

All tests in `ant_simulation` (core crate, no Bevy dependency):

| Test file | Tests |
|-----------|-------|
| grid.rs | Material is_terrain, is_solid, is_diggable, dig_ticks, GridPos neighbor/cardinals/boundary, PheromoneLayer default, Cell new defaults, Grid new/size/index/contains/get/set/cell_below/surface_y/iter_positions, generate_initial_world |
| terrain.rs | start_dig dirt/stone/duplicate, process_digging single/step/wet_dirt, stability collapse/supported |
| tick.rs | speed intervals/tps, simulation new/tick/day/speed/paused |
| snapshot.rs | snapshot from simulation, cell snapshot matches grid |
| persistence.rs | save/load roundtrip, version, invalid bytes error |

Run with: `cargo test -p ant_simulation`
