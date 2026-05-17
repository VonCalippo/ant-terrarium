# Queen & Life Cycle — Design Spec

**Date**: 2026-05-17
**Status**: Approved
**Part of**: Ant Terrarium Simulator (Sub-project 4 of 6)
**Depends on**: Sub-project 1 (Grid), 2 (Ant AI), 3 (Pheromones)

## Overview

Transform the queen from a decorative marker into a biological entity. The queen lays eggs, which hatch into larvae, pupate, and emerge as adult workers. Reproduction rate depends on food delivered to the queen by workers. Ants age and eventually die. Death produces corpses that decay and emit death pheromone.

## Queen Entity

The queen becomes an active entity (not just a golden dot):

```rust
pub struct Queen {
    pub pos: GridPos,
    pub health: f32,           // 0.0-1.0, 0 = dead
    pub stress: f32,           // 0.0-1.0
    pub hunger: f32,           // 0.0-1.0
    pub egg_progress: f32,     // 0.0-1.0, when 1.0 → lay egg
    pub food_reserve: u16,     // food units stored (from worker deliveries)
    pub alive: bool,
}
```

### Queen needs
- **Hunger**: increases 0.001/tick. When >0.7, egg production slows. When >0.9, stops.
- **Stress**: increases from overcrowding, danger pheromones, lack of food. When >0.8, egg production slows.
- **Food reserve**: incremented when workers deliver food nearby (+1 per delivery). Consumed to produce eggs (-5 per egg).
- **Health**: decays when hunger >0.9 or stress >0.9. Recovers when well-fed and calm.

### Queen death
When health reaches 0:
- Queen entity removed
- Massive DEATH pheromone burst (200) at queen position
- All ants get +0.5 stress, +0.3 fear
- No more eggs laid
- Colony slowly collapses

## Life Cycle

### Stages
```
Egg ──(600 ticks)──→ Larva ──(600 ticks)──→ Pupa ──(600 ticks)──→ Adult Worker
```

Each stage occupies a grid cell near the queen (or wherever the egg was laid). Larvae require:
- Temperature between 15°C-30°C (ambient default 22°C is fine)
- Humidity > 20% (ambient is fine)
- Proximity to queen (QUEEN pheromone) for faster development

### Egg laying
- Queen checks `food_reserve >= 5` and `hunger < 0.7` and `stress < 0.8`
- If conditions met, `egg_progress` advances (rate depends on food availability)
- When `egg_progress >= 1.0`, an Egg material cell appears adjacent to queen
- `food_reserve -= 5`, `egg_progress = 0`

### Worker feeding of queen
- Workers carrying food near the queen get the impulse to deliver (already exists in ant AI: `CarryFood { to: home }`)
- When a worker delivers food near the queen: `queen.food_reserve += 1`, `queen.hunger -= 0.3`
- Workers near queen may also get `FeedQueen` action when queen hunger > 0.5

### Larva care
- Workers with high maintenance_drive near larvae get impulse to feed/clean them
- This accelerates larva development by 1.5x

## Ant Aging and Death

```
pub struct AntAge {
    pub age: u32,               // ticks alive
    pub max_age: u32,           // natural lifespan (varies per ant)
    pub health: f32,            // 0.0-1.0
}
```

- Each ant gets `max_age` randomized: 50,000-80,000 ticks (~7-11 hours at 2 tick/sec)
- Health starts at 1.0, decays slowly after age > max_age * 0.8
- At health = 0, ant dies:
  - Cell becomes `OrganicWaste` material
  - DEATH pheromone burst (200) at death position
  - Nearby ants get +0.3 fear

## Corpse decay
- `OrganicWaste` cells emit DEATH pheromone at rate 5/tick
- After 2400 ticks (20 min), `OrganicWaste` → `Air` (fully decayed)

## Initial colony setup
- 1 Queen (alive), 5 worker ants (aged 0-2000 ticks random)
- No eggs/larvae/pupae initially

## What's NOT in v1
- Queen movement (she stays in place)
- Genetic inheritance of traits
- Colony splitting / new queens
