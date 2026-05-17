# Core World & Rendering — Design Spec

**Date**: 2026-05-17  
**Status**: Approved  
**Part of**: Ant Terrarium Simulator (Sub-project 1 of 6)

## Overview

First sub-project of the Ant Terrarium Simulator. Delivers the 2D grid world, terrain physics, rendering, UI, and persistence. No ant entities or AI yet. The queen is rendered as a decorative golden dot at the surface center — no Queen struct, no behavior, no egg-laying. She marks the colony origin point.

## Technology

- **Language**: Rust (stable)
- **Engine**: Bevy 0.15+ (2D only: `bevy_sprite`, `bevy_ui`, `bevy_input`)
- **Serialization**: `serde` + `bincode`
- **Architecture**: Two-crate workspace — `ant_simulation` (pure Rust, no Bevy) + `ant_renderer` (Bevy plugin)

## Project Structure

```
ant/
├── Cargo.toml              # workspace
├── crates/
│   ├── ant_simulation/     # Core simulazione (no Bevy)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── grid.rs          # Grid + tipi cella
│   │       ├── terrain.rs       # Fisica terreno (scavo, stabilità)
│   │       ├── tick.rs          # Loop simulazione
│   │       ├── snapshot.rs      # Stato esportabile per rendering
│   │       └── persistence.rs   # Save/load bincode
│   │
│   └── ant_renderer/      # Bevy frontend
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs           # Plugin Bevy
│           ├── app.rs           # Configurazione app
│           ├── camera.rs        # Camera 2D, zoom, pan
│           ├── sprites.rs       # Render celle come sprite
│           ├── hud.rs           # Pannello info, controlli velocità
│           ├── input.rs         # Mouse/tastiera → comandi
│           └── assets.rs        # Palette colori, texture minime
│
└── src/
    └── main.rs             # Entry point: avvia Bevy + simulation core
```

## Data Flow

```
ant_simulation::tick() → Snapshot → ant_renderer::apply_snapshot()
ant_renderer::input()  → Command  → ant_simulation::apply_command()
```

Two simple structs form the interface: `Snapshot` (sim → renderer) and `Command` (renderer → sim).

## Grid Data Model

```rust
pub struct Grid {
    pub width: u16,          // 128
    pub height: u16,         // 96
    pub cells: Vec<Cell>,    // flat: [y * width + x]
}

pub struct Cell {
    pub material: Material,
    pub humidity: u8,        // 0-255
    pub temperature: i16,    // decimi di grado (-100.0..100.0°C)
    pub stability: u8,       // 0=collapsed, 255=bedrock
    pub pheromones: PheromoneLayer,  // reserved for sub-project 3
    pub organic_matter: u8,
}

pub enum Material {
    Air, LooseDirt, Dirt, WetDirt, Sand, Stone, Water,
    Food, OrganicWaste, Fungus, Egg, Larva,
}
```

- Flat vec: single allocation, cache-friendly, direct serialization
- No chunking at 128×96 (~12K cells); architecture supports it later
- `GridPos { x: u16, y: u16 }` for coordinates
- Grid exposes neighbor queries, rectangular area queries, and iteration

## Terrain Physics

### Digging (multi-step)

```
Stone   → (not diggable by workers)
Dirt    → LooseDirt (1 tick) → Air
Sand    → Air (instant, 0 ticks)
WetDirt → LooseDirt (2 ticks, slower)
```

```rust
pub struct DigState {
    pub target: GridPos,
    pub ticks_remaining: u8,
}
```

### Structural Stability

Rule: a terrain cell (Dirt, Sand, WetDirt, LooseDirt) without support underneath collapses.

```
fn update_stability(grid):
  For each terrain cell:
    if cell_below is Air or Water:
      stability -= decay_rate
      if stability == 0: collapse to LooseDirt, then Air
    if cell_below is solid:
      stability = min(255, stability + recovery)
```

Stability values: Stone=255 (immovable), Dirt=128 (stable), LooseDirt=64 (fragile), Air=0.

Decay: Dirt (128) without support collapses in ~32 ticks (~8 real seconds at 4 tick/sec).

### Terrain Events

```rust
pub enum TerrainEvent {
    DigProgress { pos: GridPos, remaining: u8 },
    DigComplete { pos: GridPos },
    Collapse { pos: GridPos, from: Material, to: Material },
    CollapseChain { positions: Vec<GridPos> },
}
```

### Out of scope for v1
- Dynamic water (flow, absorption)
- Variable humidity (static initial values)
- Fungus growth, decomposition

## Tick Engine

```rust
pub struct Simulation {
    pub grid: Grid,
    pub tick: u64,
    pub speed: Speed,
    pub events: Vec<TerrainEvent>,
    pub pending_digs: Vec<DigState>,
}

pub enum Speed { Paused, Normal, Fast, Fastest }

impl Simulation {
    pub fn tick(&mut self) -> TickResult {
        self.tick += 1;
        self.events.clear();
        self.process_digging();
        self.events.extend(self.grid.update_stability());
        TickResult { tick: self.tick, events: self.events.clone() }
    }
}
```

### Timing

| Speed | Tick rate | Real interval |
|-------|-----------|---------------|
| Paused | 0 | — |
| Normal (1x) | 2/sec | 500ms |
| Fast (2x) | 4/sec | 250ms |
| Fastest (4x) | 10/sec | 100ms |

The renderer uses `std::time::Instant` to accumulate wall-clock time and call `simulation.tick()` at the correct interval. Simulation is deterministic given the same tick count and RNG seed.

## Persistence

### Format

Bincode via `serde`. Save file structure:

```rust
struct SaveFile {
    version: u32,
    tick: u64,
    width: u16,
    height: u16,
    cells: Vec<SavedCell>,
    rng_state: [u8; 32],
}
```

~120KB per save at 128×96 (~10 bytes per cell).

### Behavior

- **Auto-save**: every 60 real seconds (configurable), non-blocking
- **Manual save**: key S or menu action
- **On startup**: load latest autosave; if none, generate fresh world
- **Location**: `%APPDATA%/ant_terrarium/saves/`
- **Atomic writes**: write to temp file, then rename

## Rendering

### Style
Pixel-art minimal (Phase A). Architecture supports illustrative style (Phase B) later. Each cell is a 1×1 pixel sprite scaled to 8×8 window pixels.

### Palette

| Material | Color |
|----------|-------|
| Air | #3A4A6A (dark blue-gray) |
| Dirt | #8B6914 (earthy brown) |
| LooseDirt | #AD8B3C (lighter brown) |
| Surface | #5A7A3A (mossy green) |
| Stone | #686868 (medium gray) |
| Sand | #C2B280 (tan) |
| Water | #2A5A8A (blue) |
| Food | #E8C040 (amber) |
| Queen | #E8C040 (golden dot) |

### Terrarium View
- Side cross-section: sky at top, surface layer, dirt strata, stone bedrock at bottom
- Glass frame: dark border (3px) with subtle blue reflection gradient overlay (top 30%)
- Particles: temporary entities with fade-out for dig/collapse events

### Camera
- Orthographic 2D camera
- Zoom: scroll wheel, range 0.5x–4x, centered on cursor
- Pan: right-click drag (with inertia) or WASD keys
- Snap: double-click centers on queen or clicked cell

## UI / HUD

### Bottom Bar (always visible)
```
[⏸] [▶] [⏩]  |  Tick: 12,847  |  Day: 3  |  Queen: ●  |  Workers: 0
```

1 sim-day ≈ 2400 ticks ≈ 20 real minutes at 2 tick/sec.

### Side Panel (toggle with Tab)
Semi-transparent overlay at right edge showing:
- Queen marker (present at origin — decorative only, no logic)
- Population (eggs, larvae, workers — all 0 in this phase)
- Environment (avg humidity, temperature, tunnels dug, collapses)
- Action buttons: Deposit Food, Add Water, Save, Load

### Controls

| Action | Input |
|--------|-------|
| Deposit food | Left-click on any Air cell at the surface (y=0 row in world coords) |
| Add water | Right-click on a Dirt/Sand cell to turn it into WetDirt |
| Modify terrain | Shift+click on cells in the top 3 rows (surface zone) to toggle Dirt↔Air |
| Zoom | Scroll wheel |
| Pan | Middle-click drag or WASD |
| Pause/Play | Space |
| Speed 1x/2x/4x | Keys 1, 2, 3 |
| Toggle panel | Tab |
| Save | Ctrl+S |

## Initial World

Empty terrarium with flat terrain surface, a single visible queen on the surface, no tunnels, no workers. The colony starts from absolute zero.

## What's NOT in This Sub-Project
- Ant entities, AI, movement, or behavior (sub-project 2)
- Pheromone system (sub-project 3)
- Queen reproduction and life cycle (sub-project 4)
- Dynamic ecology, water flow, fungus (sub-project 5)
- Desktop widget chrome, window management (sub-project 6 — or integrated into rendering as needed)
