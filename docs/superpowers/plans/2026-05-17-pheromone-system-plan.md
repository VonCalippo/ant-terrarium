# Pheromone System — Implementation Plan

> **For agentic workers:** Implement directly — tightly coupled changes across ant.rs, grid.rs, tick.rs.

**Goal:** Activate pheromone deposition, evaporation, and decision influence. No spatial diffusion.

**Architecture:** PheromoneLayer already exists in grid.rs. Add evaporation pass in tick loop, pheromone deposition in ant actions, pheromone perception in ant perception, and impulse modifiers in decision-making.

**Tech Stack:** Rust, existing ant_simulation crate (no Bevy changes needed for core).

---

## File Map
```
crates/ant_simulation/src/
├── grid.rs              # PheromoneLayer exists; add evaporation method
├── ant.rs               # Modify: deposition in actions, perception extension, impulse modifiers
├── tick.rs              # Modify: call evaporation in tick loop
└── persistence.rs       # Already handles PheromoneLayer — no changes needed
```

---

### Task 1: Add evaporation system to Grid

**Files:** `crates/ant_simulation/src/grid.rs`

Add to `Grid`:
```rust
pub fn evaporate_pheromones(&mut self) {
    const RATE: u8 = 1;
    for cell in &mut self.cells {
        cell.pheromones.food = cell.pheromones.food.saturating_sub(RATE);
        cell.pheromones.home = cell.pheromones.home.saturating_sub(RATE);
        cell.pheromones.danger = cell.pheromones.danger.saturating_sub(RATE);
        cell.pheromones.dig = cell.pheromones.dig.saturating_sub(RATE);
        cell.pheromones.queen = cell.pheromones.queen.saturating_sub(RATE);
        cell.pheromones.death = cell.pheromones.death.saturating_sub(RATE);
        cell.pheromones.waste = cell.pheromones.waste.saturating_sub(RATE);
    }
}

pub fn deposit_pheromone(&mut self, pos: GridPos, ptype: PheromoneType, amount: u8) {
    if let Some(cell) = self.get_mut(pos) {
        let target = match ptype {
            PheromoneType::Food => &mut cell.pheromones.food,
            PheromoneType::Home => &mut cell.pheromones.home,
            PheromoneType::Danger => &mut cell.pheromones.danger,
            PheromoneType::Dig => &mut cell.pheromones.dig,
            PheromoneType::Queen => &mut cell.pheromones.queen,
            PheromoneType::Death => &mut cell.pheromones.death,
            PheromoneType::Waste => &mut cell.pheromones.waste,
        };
        *target = target.saturating_add(amount);
    }
}
```

```rust
pub enum PheromoneType { Food, Home, Danger, Dig, Queen, Death, Waste }
```

### Task 2: Call evaporation in tick loop

**Files:** `crates/ant_simulation/src/tick.rs`

In `Simulation::tick()`:
```rust
self.grid.evaporate_pheromones();
```

### Task 3: Add pheromone deposition to ant actions

**Files:** `crates/ant_simulation/src/ant.rs`

Modify `execute_action` to deposit on relevant actions. After each action, call:
```rust
grid.deposit_pheromone(body.pos, PheromoneType::*, amount);
```

| Action | Trigger | Type | Amount |
|--------|---------|------|--------|
| CollectFood (on food cell) | Before collection | Food | 200 |
| Move while carrying food | Each move step | Food | 30 |
| Flee | On flee cell | Danger | 150 |
| Idle near danger | After perception | Danger | 40 |
| Dig | On target cell | Dig | 100 |
| Rest near queen | After rest complete | Queen | 50 |
| CarryDirt dump | On dump cell | Waste | 60 |

### Task 4: Extend perception to read pheromones

**Files:** `crates/ant_simulation/src/ant.rs`

In `perceive()`, read pheromone layers from 3×3 neighborhood:
```rust
pub pheromones: [[PheromoneLayer; 3]; 3],
```

Read from grid: `grid.get(np).map(|c| c.pheromones).unwrap_or_default()`

Add to `LocalPerception`:
```rust
pub fn strongest_pheromone_dir(&self, ptype: PheromoneType) -> Option<(Direction, u8)> {
    // Check 8 surrounding cells, return direction of strongest pheromone
}
```

### Task 5: Add pheromone influence to decision-making

**Files:** `crates/ant_simulation/src/ant.rs`

In `calculate_impulses()`, add pheromone-driven impulses:

```rust
// FOOD pheromone attraction
if let Some((dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Food) {
    let w = (strength as f32 / 255.0) * 0.6 * (1.0 + traits.pheromone_sensitivity);
    impulses.push(Impulse { action: Action::Move(dir), weight: w });
}

// DANGER pheromone repel
if let Some((dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Danger) {
    let away = opposite_dir(dir);
    let w = (strength as f32 / 255.0) * 0.7 * (1.0 + traits.pheromone_sensitivity);
    impulses.push(Impulse { action: Action::Flee { from: body.pos }, weight: w });
    brain.fear = (brain.fear + strength as f32 * 0.005).min(1.0);
}

// DIG pheromone attraction
if let Some((dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Dig) {
    let w = (strength as f32 / 255.0) * 0.3 * (1.0 + traits.pheromone_sensitivity);
    impulses.push(Impulse { action: Action::Dig(dir), weight: w });
}

// QUEEN pheromone calming
let q_strength = perception.pheromones[1][1].queen as f32 / 255.0;
brain.stress = (brain.stress - q_strength * 0.02).max(0.0);

// DEATH pheromone repel
if let Some((dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Death) {
    let away = opposite_dir(dir);
    let w = (strength as f32 / 255.0) * 0.6;
    brain.fear = (brain.fear + strength as f32 * 0.008).min(1.0);
    impulses.push(Impulse { action: Action::Move(away), weight: w });
}
```

### Task 6: Tests and verification

```rust
#[test]
fn test_pheromone_evaporation() {
    let mut grid = Grid::new(10, 10);
    grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 200);
    assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().pheromones.food, 200);
    grid.evaporate_pheromones();
    assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().pheromones.food, 199);
}

#[test]
fn test_pheromone_deposit_saturates() {
    let mut grid = Grid::new(10, 10);
    grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 250);
    grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 20);
    assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().pheromones.food, 255);
}
```
