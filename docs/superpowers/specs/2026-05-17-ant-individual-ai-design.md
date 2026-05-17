# Ant Individual AI — Design Spec

**Date**: 2026-05-17
**Status**: Approved
**Part of**: Ant Terrarium Simulator (Sub-project 2 of 6)
**Depends on**: Sub-project 1 (Core World & Rendering)

## Overview

Implement individual ant entities with needs-based decision making, limited memory, unique personality traits, local perception, and discrete movement on the grid. Population: 5 worker ants spawned at simulation start near the queen. No reproduction yet.

## Technology
- Rust + Bevy 0.15 (ECS)
- `ant_simulation` crate extended with ant AI logic
- `ant_renderer` crate extended with ant sprites

## ECS Components

### AntBody — physics and current state
```rust
#[derive(Component)]
pub struct AntBody {
    pub pos: GridPos,
    pub direction: Direction,
    pub current_action: Action,
    pub action_ticks: u8,
    pub carrying: Option<CarriedItem>,
}
```

### AntBrain — internal state
```rust
#[derive(Component)]
pub struct AntBrain {
    pub hunger: f32,
    pub fatigue: f32,
    pub fear: f32,
    pub social_drive: f32,
    pub exploration_drive: f32,
    pub maintenance_drive: f32,
    pub stress: f32,
    pub agitation: f32,
}
```

### AntMemory — spatial recall
```rust
#[derive(Component)]
pub struct AntMemory {
    pub last_positions: VecDeque<GridPos>,  // max 32
    pub nest_direction: Option<Direction>,
    pub recent_food: Vec<GridPos>,          // max 8
    pub recent_dangers: Vec<GridPos>,       // max 4
    pub home_position: GridPos,
}
```

### AntTraits — individual variation
```rust
#[derive(Component)]
pub struct AntTraits {
    pub curiosity: f32,              // 0.0-1.0
    pub aggression: f32,             // 0.0-1.0
    pub pheromone_sensitivity: f32,  // 0.0-1.0
    pub chaos_tolerance: f32,        // 0.0-1.0
    pub efficiency: f32,             // 0.0-1.0
    pub speed_modifier: f32,         // 0.7-1.3
}
```

Generated randomly with Gaussian distribution around 0.5, clamped to 0.0-1.0. `speed_modifier` is a multiplier on action duration.

## Actions

```rust
pub enum Action {
    Idle,
    Move(Direction),
    Dig(Direction),
    CarryDirt { to: GridPos },
    CollectFood,
    CarryFood { to: GridPos },
    Eat,
    Rest,
    Groom,
    Flee { from: GridPos },
}
```

Action duration (ticks, modified by `speed_modifier`):
| Action | Base ticks |
|--------|------------|
| Move | 1 |
| Dig | 7 (Dirt) |
| CollectFood | 3 |
| Eat | 5 |
| Rest | 10 |
| Carry* | 1 per cell |
| Groom | 4 |
| Flee | 1 |

## Decision Making (Utility AI)

### Per-tick loop
1. **Update needs**: hunger += 0.003, fatigue += 0.002, fear decays to 0, etc.
2. **Perceive local environment** (3×3 neighborhood)
3. **Calculate impulses** from needs × perception × traits
4. **Select action**: highest-weighted impulse, with 5% random deviation chance
5. **Execute action** (progress or complete)

### Needs → Impulse mapping
| Need | Condition | Impulse |
|------|-----------|---------|
| hunger > 0.6 | food in perception | CollectFood (0.8) |
| hunger > 0.3 | food in perception | CollectFood (0.4) |
| hunger > 0.8 | no food visible | Move(random) (0.7) — explore |
| fatigue > 0.5 | safe | Rest (0.6) |
| fear > 0.5 | danger nearby | Flee(away from danger) (0.9) |
| fear > 0.3 | crowded | Move(away) (0.5) |
| exploration < 0.4 | idle | Move(random unexplored) (0.5) |
| social > 0.5 | fewer than 2 ants nearby | Move(toward nearest ant) (0.4) |
| maintenance > 0.7 | dirt nearby | Dig (0.7) |
| maintenance > 0.4 | carrying dirt | CarryDirt(to dump) (0.6) |
| carrying food | near home | CarryFood(to home) (0.9) |
| stress > 0.7 | any | Groom (0.5) |

### Trait modifiers
- `curiosity` → exploration impulse weight × (0.5 + trait)
- `aggression` → Flee impulse weight × (1.0 - trait)
- `chaos_tolerance` → stress accumulation rate × (1.0 - trait)
- `efficiency` → action ticks multiplier
- `speed_modifier` → action ticks multiplier

## Local Perception

```rust
struct LocalPerception {
    cells: [[Cell; 3]; 3],      // 3x3 grid centered on ant
    nearby_ants: Vec<(Direction, u8)>,  // direction + count
    pheromones: [[PheromoneLayer; 3]; 3],  // reserved for sub-project 3
    food_detected: bool,
    danger_detected: bool,
    queen_detected: bool,
}
```

## Movement

- Discrete, cell-per-cell, 8 cardinal+diagonal directions
- 1 cell per Move tick (modified by speed_modifier)
- Collision avoidance: if target cell occupied, try alternate directions; if all blocked, skip tick
- Oscillation: after 3+ consecutive moves in same direction, 10% chance to deviate ±45°
- Trail memory: avoids revisiting last 4 positions unless no alternative

## Initial Spawn

5 ants spawned at tick 0 at random positions within 3 cells of the queen. Queen is the decorative golden dot at `grid.queen_position()`.

## Out of Scope
- Pheromone deposition/detection (sub-project 3)
- Reproduction, eggs, larvae, queen biology (sub-project 4)
- Digging execution (ants can initiate dig but terrain physics is from sub-project 1)
- Complex pathfinding, global navigation
