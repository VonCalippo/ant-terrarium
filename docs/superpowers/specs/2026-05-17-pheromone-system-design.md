# Pheromone System — Design Spec

**Date**: 2026-05-17
**Status**: Approved
**Part of**: Ant Terrarium Simulator (Sub-project 3 of 6)
**Depends on**: Sub-project 1 (Grid), Sub-project 2 (Ant AI)

## Overview

Implement the pheromone system — the colony's chemical nervous system. Pheromones are deposited by ants, evaporate over time, and influence ant decision-making via local gradients. No spatial diffusion in v1: gradients emerge from repeated ant traffic.

## Pheromone Types

```rust
pub struct PheromoneLayer {
    pub food: u8,      // 0-255
    pub home: u8,
    pub danger: u8,
    pub dig: u8,
    pub queen: u8,
    pub death: u8,
    pub waste: u8,
}
```

This already exists in `grid.rs`. Previously reserved — now active.

## Deposition

Ants deposit pheromones as a side effect of actions:

| Action | Pheromone | Strength | Condition |
|--------|-----------|----------|-----------|
| CollectFood | FOOD | 200 | On the food cell before collection |
| Move (carrying food) | FOOD | 30 | Each cell traversed while carrying food |
| Move (returning home) | HOME | 80 | When ant has food and is near home |
| Flee | DANGER | 150 | On the cell being fled from |
| Idle near danger | DANGER | 40 | Passive fear emission |
| Dig | DIG | 100 | On the dirt cell being dug |
| Rest near queen | QUEEN | 50 | Passive queen proximity emission |
| Die | DEATH | 200 | On death cell |
| CarryDirt / dump | WASTE | 60 | On the dump site |

Deposition writes directly to `grid.cells[idx].pheromones.<type>`.

## Evaporation

Every tick, all pheromones on all cells decay:

```rust
const EVAPORATION_RATE: u8 = 1; // per tick, per pheromone type
```

A cell with 200 FOOD pheromone takes 200 ticks (~50 seconds at 4 tick/sec) to fully evaporate. Strong enough to persist between ant visits, weak enough to fade unused trails.

## Ant Perception of Pheromones

Extended `LocalPerception` (already in `ant.rs`) to include pheromone readings from the 3×3 neighborhood:

```rust
pub struct LocalPerception {
    // ... existing fields ...
    pub pheromones: [[PheromoneLayer; 3]; 3],
}
```

## Pheromone Influence on Decision-Making

New impulse modifiers based on pheromone gradients:

| Pheromone | Effect | Weight modifier |
|-----------|--------|----------------|
| FOOD (strong) | Attract toward gradient | +0.6 to Move(toward food) |
| FOOD (weak) | Slight attraction | +0.2 to exploration in that direction |
| HOME (strong) | Attract toward gradient | +0.5 to Move(toward home) |
| DANGER | Repel from gradient | +0.7 to Flee(away), +0.4 to fear |
| DIG | Attract toward gradient | +0.3 to Dig in that direction |
| QUEEN | Calming effect | -0.2 stress per tick near queen pheromone |
| DEATH | Strong repel | +0.6 to fear, +0.3 to avoidance |
| WASTE | Mild repel | +0.2 to avoidance |

Ants with higher `pheromone_sensitivity` trait get stronger modifiers (×1.0 to ×2.0 multiplier).

## Integration with Existing Systems

- **Grid**: `PheromoneLayer` already exists in `Cell`. Activate evaporation in `tick()`.
- **Ants**: `LocalPerception` extended. `calculate_impulses` modified. `execute_action` deposits.
- **Rendering**: Optional — render pheromones as subtle color overlays on cells (deferred to later if complex).
- **Persistence**: `PheromoneLayer` already serialized in `SavedCell`. Works automatically.

## Out of Scope
- Spatial diffusion (may add later if gradients too sparse)
- Pheromone rendering (can be added later)
- Pheromone-specific ant memory beyond what exists
