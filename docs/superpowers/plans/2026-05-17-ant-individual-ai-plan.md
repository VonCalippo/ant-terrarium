# Ant Individual AI — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement 5 worker ants with needs-based utility AI, limited memory, individual traits, local perception, and discrete grid movement.

**Architecture:** New `ant.rs` module in `ant_simulation` defines components and systems. Ants are Bevy entities spawned at simulation start. A new `ants.rs` module in `ant_renderer` handles ant sprite rendering. Decision-making is pure utility AI with trait-modified impulse weights.

**Tech Stack:** Rust stable, Bevy 0.15 ECS, ant_simulation (extended), ant_renderer (extended)

---

## File Map

```
crates/ant_simulation/src/
├── lib.rs              # + pub mod ant
├── ant.rs              # NEW — AntBody, AntBrain, AntMemory, AntTraits, Action, Impulse, perception, decision, movement
├── grid.rs             # unchanged
├── terrain.rs          # unchanged
├── tick.rs             # modified — spawn ants on init, tick ant systems
├── snapshot.rs         # modified — add ant data to snapshot
└── persistence.rs      # modified — save/load ant state

crates/ant_renderer/src/
├── app.rs              # modified — register ant systems
├── sprites.rs          # modified — render ant sprites
├── ants.rs             # NEW — ant sprite spawning and updating
└── assets.rs           # modified — ant color palette
```

---

## Phase 1: Core ant components and logic (ant_simulation)

### Task 1: Define Action enum and ECS component structs

**Files:**
- Create: `crates/ant_simulation/src/ant.rs`

```rust
use serde::{Serialize, Deserialize};
use crate::grid::{GridPos, Direction, Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl Action {
    pub fn base_ticks(self) -> u8 {
        match self {
            Action::Move(_) => 1,
            Action::Dig(_) => 7,
            Action::CollectFood => 3,
            Action::Eat => 5,
            Action::Rest => 10,
            Action::CarryDirt { .. } | Action::CarryFood { .. } => 1,
            Action::Groom => 4,
            Action::Flee { .. } => 1,
            Action::Idle => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CarriedItem {
    Dirt,
    Food,
}

#[derive(Debug, Clone)]
pub struct AntBody {
    pub pos: GridPos,
    pub direction: Direction,
    pub current_action: Action,
    pub action_ticks: u8,
    pub carrying: Option<CarriedItem>,
}

impl AntBody {
    pub fn new(pos: GridPos) -> Self {
        Self {
            pos,
            direction: Direction::S,
            current_action: Action::Idle,
            action_ticks: 0,
            carrying: None,
        }
    }
}

#[derive(Debug, Clone)]
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

impl Default for AntBrain {
    fn default() -> Self {
        Self {
            hunger: 0.0,
            fatigue: 0.0,
            fear: 0.0,
            social_drive: 0.5,
            exploration_drive: 0.5,
            maintenance_drive: 0.3,
            stress: 0.0,
            agitation: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AntMemory {
    pub last_positions: Vec<GridPos>,     // max 32
    pub nest_direction: Option<Direction>,
    pub recent_food: Vec<GridPos>,        // max 8
    pub recent_dangers: Vec<GridPos>,     // max 4
    pub home_position: GridPos,
}

impl AntMemory {
    pub fn new(home: GridPos) -> Self {
        Self {
            last_positions: Vec::with_capacity(32),
            nest_direction: None,
            recent_food: Vec::with_capacity(8),
            recent_dangers: Vec::with_capacity(4),
            home_position: home,
        }
    }

    pub fn push_position(&mut self, pos: GridPos) {
        if self.last_positions.len() >= 32 {
            self.last_positions.remove(0);
        }
        self.last_positions.push(pos);
    }

    pub fn recently_visited(&self, pos: GridPos, n: usize) -> bool {
        let len = self.last_positions.len();
        let start = len.saturating_sub(n);
        self.last_positions[start..].contains(&pos)
    }
}

#[derive(Debug, Clone)]
pub struct AntTraits {
    pub curiosity: f32,
    pub aggression: f32,
    pub pheromone_sensitivity: f32,
    pub chaos_tolerance: f32,
    pub efficiency: f32,
    pub speed_modifier: f32,
}

impl AntTraits {
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        Self {
            curiosity: random_trait(rng),
            aggression: random_trait(rng),
            pheromone_sensitivity: random_trait(rng),
            chaos_tolerance: random_trait(rng),
            efficiency: random_trait(rng),
            speed_modifier: 0.7 + rng.gen::<f32>() * 0.6,
        }
    }
}

fn random_trait(rng: &mut impl rand::Rng) -> f32 {
    (rng.gen::<f32>() * 0.3 + 0.35).clamp(0.0, 1.0)
}
```

- [ ] **Step 1: Run tests:** `cargo test -p ant_simulation`
- [ ] **Step 2: Commit:** `git commit -m "feat: add Action enum and ant ECS components"`

---

### Task 2: Define impulse system and decision-making

**Files:**
- Modify: `crates/ant_simulation/src/ant.rs` (add decision logic)

```rust
#[derive(Debug, Clone)]
pub struct Impulse {
    pub action: Action,
    pub weight: f32,
}

pub struct LocalPerception {
    pub cells: [[Material; 3]; 3],
    pub food_detected: bool,
    pub food_positions: Vec<(i8, i8)>,  // relative coords
    pub danger_detected: bool,
    pub danger_positions: Vec<(i8, i8)>,
    pub nearby_ant_count: u8,
    pub nearest_ant_dir: Option<Direction>,
    pub queen_detected: bool,
    pub queen_dir: Option<Direction>,
    pub dirt_adjacent: Vec<Direction>,
}

pub fn perceive(
    grid: &crate::grid::Grid,
    pos: GridPos,
    home: GridPos,
    _all_ants: &[(GridPos, Direction)],  // placeholder for ant positions
) -> LocalPerception {
    let mut cells = [[Material::Air; 3]; 3];
    let mut food_positions = Vec::new();
    let mut danger_positions = Vec::new();
    let mut food_detected = false;
    let mut danger_detected = false;
    let mut dirt_adjacent = Vec::new();
    let mut nearby_ant_count: u8 = 0;
    let mut nearest_ant_dir = None;
    let mut queen_detected = false;
    let mut queen_dir = None;

    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 { continue; }
            let nx = pos.x as i32 + dx as i32;
            let ny = pos.y as i32 + dy as i32;
            if nx < 0 || ny < 0 { continue; }
            let np = GridPos::new(nx as u16, ny as u16);
            if let Some(cell) = grid.get(np) {
                let sx = (dx + 1) as usize;
                let sy = (dy + 1) as usize;
                cells[sy][sx] = cell.material;

                match cell.material {
                    Material::Food | Material::OrganicWaste => {
                        food_detected = true;
                        food_positions.push((dx, dy));
                    }
                    Material::Fungus => {
                        danger_detected = true;
                        danger_positions.push((dx, dy));
                    }
                    Material::Dirt | Material::WetDirt | Material::Sand => {
                        dirt_adjacent.push(match (dx, dy) {
                            (0, -1) => Direction::N, (0, 1) => Direction::S,
                            (1, 0) => Direction::E, (-1, 0) => Direction::W,
                            (1, -1) => Direction::NE, (-1, -1) => Direction::NW,
                            (1, 1) => Direction::SE, (-1, 1) => Direction::SW,
                            _ => continue,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    // Detect queen proximity
    let dx = home.x as i32 - pos.x as i32;
    let dy = home.y as i32 - pos.y as i32;
    let dist = ((dx * dx + dy * dy) as f32).sqrt();
    if dist <= 3.0 && dist > 0.0 {
        queen_detected = true;
        queen_dir = Some(approx_direction(dx, dy));
    }

    LocalPerception {
        cells,
        food_detected,
        food_positions,
        danger_detected,
        danger_positions,
        nearby_ant_count,
        nearest_ant_dir,
        queen_detected,
        queen_dir,
        dirt_adjacent,
    }
}

fn approx_direction(dx: i32, dy: i32) -> Direction {
    use Direction::*;
    if dx.abs() > dy.abs() * 2 { return if dx > 0 { E } else { W }; }
    if dy.abs() > dx.abs() * 2 { return if dy > 0 { S } else { N }; }
    match (dx > 0, dy > 0) {
        (true, true) => SE, (true, false) => NE,
        (false, true) => SW, (false, false) => NW,
    }
}

pub fn calculate_impulses(
    brain: &AntBrain,
    memory: &AntMemory,
    traits: &AntTraits,
    perception: &LocalPerception,
    body: &AntBody,
) -> Vec<Impulse> {
    let mut impulses = Vec::new();

    // Hunger drives
    if perception.food_detected && brain.hunger > 0.3 {
        let w = if brain.hunger > 0.6 { 0.8 } else { 0.4 };
        impulses.push(Impulse { action: Action::CollectFood, weight: w });
    }
    if brain.hunger > 0.8 && !perception.food_detected {
        impulses.push(Impulse {
            action: Action::Move(random_direction_except(&memory.last_positions)),
            weight: 0.7,
        });
    }
    if brain.hunger > 0.0 && body.carrying == Some(CarriedItem::Food) && perception.queen_detected {
        impulses.push(Impulse {
            action: Action::CarryFood { to: memory.home_position },
            weight: 0.9,
        });
    }
    if brain.hunger > 0.0 && body.carrying == Some(CarriedItem::Food) {
        let dir = memory.nest_direction.unwrap_or(perception.queen_dir.unwrap_or(Direction::S));
        impulses.push(Impulse { action: Action::Move(dir), weight: 0.4 });
    }

    // Fatigue
    if brain.fatigue > 0.5 && !perception.danger_detected {
        impulses.push(Impulse { action: Action::Rest, weight: brain.fatigue.min(0.8) });
    }

    // Fear → flee
    if perception.danger_detected && brain.fear > 0.3 {
        if let Some((dx, dy)) = perception.danger_positions.first() {
            let away = approx_direction(-*dx as i32, -*dy as i32);
            let w = 0.5 + (1.0 - traits.aggression) * 0.4;
            impulses.push(Impulse { action: Action::Flee { from: body.pos }, weight: w });
        }
    }

    // Exploration
    if brain.exploration_drive > 0.6 && body.current_action == Action::Idle {
        let w = traits.curiosity * 0.6;
        impulses.push(Impulse {
            action: Action::Move(random_direction_except(&memory.last_positions)),
            weight: w,
        });
    }

    // Maintenance → digging
    if brain.maintenance_drive > 0.5 && !perception.dirt_adjacent.is_empty() {
        let dir = perception.dirt_adjacent[0];
        impulses.push(Impulse { action: Action::Dig(dir), weight: 0.2 + traits.efficiency * 0.5 });
    } else if brain.maintenance_drive < 0.3 {
        // Restore maintenance drive slowly
    }

    // Carrying dirt → dump
    if body.carrying == Some(CarriedItem::Dirt) {
        let dump_dir = find_dump_spot(&perception.cells);
        impulses.push(Impulse { action: Action::CarryDirt { to: body.pos }, weight: 0.6 });
    }

    // Stress reduction
    if brain.stress > 0.7 {
        impulses.push(Impulse { action: Action::Groom, weight: 0.5 });
    }

    // Social — if alone, move toward others
    if perception.nearby_ant_count < 2 && brain.social_drive > 0.3 {
        if let Some(dir) = perception.nearest_ant_dir {
            impulses.push(Impulse { action: Action::Move(dir), weight: 0.3 });
        }
    }

    // Always add Idle as fallback
    impulses.push(Impulse { action: Action::Idle, weight: 0.1 });

    impulses
}

fn random_direction_except(recent: &[GridPos]) -> Direction {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let dirs = &Direction::ALL;
    dirs[rng.gen_range(0..dirs.len())]
}

fn find_dump_spot(cells: &[[Material; 3]; 3]) -> Option<Direction> {
    // Find an Air cell to dump dirt
    for dy in 0..3 {
        for dx in 0..3 {
            if dx == 1 && dy == 1 { continue; }
            if cells[dy][dx] == Material::Air {
                return Some(match (dx as i8 - 1, dy as i8 - 1) {
                    (0, -1) => Direction::N, (0, 1) => Direction::S,
                    (1, 0) => Direction::E, (-1, 0) => Direction::W,
                    _ => continue,
                });
            }
        }
    }
    None
}

pub fn select_action(impulses: &[Impulse], traits: &AntTraits) -> Action {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // 5% chance of random deviation
    if rng.gen::<f32>() < traits.chaos_tolerance * 0.1 {
        return match rng.gen_range(0..4) {
            0 => Action::Idle,
            1 => Action::Move(Direction::ALL[rng.gen_range(0..8)]),
            2 => Action::Rest,
            _ => Action::Groom,
        };
    }

    impulses.iter()
        .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap())
        .map(|i| i.action)
        .unwrap_or(Action::Idle)
}
```

- [ ] **Step 1: Run tests:** `cargo test -p ant_simulation`
- [ ] **Step 2: Commit:** `git commit -m "feat: add impulse calculation and utility AI decision-making"`

---

### Task 3: Ant movement and action execution system

**Files:**
- Modify: `crates/ant_simulation/src/ant.rs` (add movement and action systems)

```rust
pub fn execute_action(
    body: &mut AntBody,
    brain: &mut AntBrain,
    memory: &mut AntMemory,
    grid: &mut crate::grid::Grid,
    tick: u64,
) -> Vec<AntEvent> {
    let mut events = Vec::new();
    let ticks_needed = (body.current_action.base_ticks() as f32 / body.action_speed_modifier(None)) as u8;
    // ... use traits for speed

    body.action_ticks += 1;

    if body.action_ticks < ticks_needed {
        return events;  // still in progress
    }

    body.action_ticks = 0;

    match body.current_action {
        Action::Move(dir) => {
            if let Some(new_pos) = body.pos.neighbor(dir) {
                if grid.contains(new_pos) {
                    if let Some(cell) = grid.get(new_pos) {
                        if cell.material.is_solid() {
                            // Can't move into solid; bounce
                            body.direction = opposite_dir(dir);
                            events.push(AntEvent::Blocked { pos: body.pos });
                        } else {
                            body.pos = new_pos;
                            body.direction = dir;
                            memory.push_position(new_pos);
                            events.push(AntEvent::Moved { from: body.pos, to: new_pos });
                        }
                    }
                }
            }
        }
        Action::Dig(dir) => {
            if let Some(target_pos) = body.pos.neighbor(dir) {
                if let Some(cell) = grid.get(target_pos) {
                    if cell.material.is_diggable() {
                        crate::terrain::start_dig(grid, target_pos, &mut Vec::new()); // pending digs managed externally
                        events.push(AntEvent::StartedDigging { pos: target_pos });
                    }
                }
            }
        }
        Action::CollectFood => {
            // Check adjacent cells for food
            for dir in &Direction::ALL {
                if let Some(pos) = body.pos.neighbor(*dir) {
                    if let Some(cell) = grid.get(pos) {
                        if cell.material == Material::Food {
                            grid.set_material(pos, Material::Air);
                            body.carrying = Some(CarriedItem::Food);
                            brain.hunger = (brain.hunger - 0.3).max(0.0);
                            events.push(AntEvent::CollectedFood { pos });
                            break;
                        }
                    }
                }
            }
        }
        Action::Eat => {
            brain.hunger = (brain.hunger - 0.5).max(0.0);
            brain.stress = (brain.stress - 0.2).max(0.0);
            events.push(AntEvent::Ate { pos: body.pos });
        }
        Action::Rest => {
            brain.fatigue = (brain.fatigue - 0.3).max(0.0);
            brain.stress = (brain.stress - 0.1).max(0.0);
            events.push(AntEvent::Rested { pos: body.pos });
        }
        Action::CarryDirt { .. } => {
            // Find adjacent Air cell to dump
            for dir in &Direction::ALL {
                if let Some(pos) = body.pos.neighbor(*dir) {
                    if let Some(cell) = grid.get(pos) {
                        if cell.material == Material::Air {
                            grid.set_material(pos, Material::LooseDirt);
                            body.carrying = None;
                            brain.maintenance_drive = (brain.maintenance_drive - 0.4).max(0.0);
                            events.push(AntEvent::DumpedDirt { pos });
                            break;
                        }
                    }
                }
            }
        }
        Action::CarryFood { to } => {
            if body.pos == to {
                body.carrying = None;
                events.push(AntEvent::DeliveredFood { pos: to });
            }
        }
        Action::Flee { from } => {
            let dx = body.pos.x as i32 - from.x as i32;
            let dy = body.pos.y as i32 - from.y as i32;
            let dir = approx_direction(dx, dy);
            if let Some(new_pos) = body.pos.neighbor(dir) {
                if grid.contains(new_pos) && grid.get(new_pos).map(|c| !c.material.is_solid()).unwrap_or(false) {
                    body.pos = new_pos;
                    memory.push_position(new_pos);
                    events.push(AntEvent::Fled { from: body.pos, to: new_pos });
                }
            }
        }
        Action::Groom => {
            brain.stress = (brain.stress - 0.3).max(0.0);
            events.push(AntEvent::Groomed { pos: body.pos });
        }
        Action::Idle => {}
    }

    events
}

fn opposite_dir(dir: Direction) -> Direction {
    match dir {
        Direction::N => Direction::S, Direction::S => Direction::N,
        Direction::E => Direction::W, Direction::W => Direction::E,
        Direction::NE => Direction::SW, Direction::SW => Direction::NE,
        Direction::NW => Direction::SE, Direction::SE => Direction::NW,
    }
}

#[derive(Debug, Clone)]
pub enum AntEvent {
    Moved { from: GridPos, to: GridPos },
    Blocked { pos: GridPos },
    StartedDigging { pos: GridPos },
    CollectedFood { pos: GridPos },
    Ate { pos: GridPos },
    Rested { pos: GridPos },
    DeliveredFood { pos: GridPos },
    DumpedDirt { pos: GridPos },
    Fled { from: GridPos, to: GridPos },
    Groomed { pos: GridPos },
}
```

- [ ] **Step 1: Run tests:** `cargo test -p ant_simulation`
- [ ] **Step 2: Commit:** `git commit -m "feat: add ant movement and action execution systems"`

---

### Task 4: Needs decay and update system

**Files:**
- Modify: `crates/ant_simulation/src/ant.rs` (add needs update)

```rust
pub fn update_needs(brain: &mut AntBrain, traits: &AntTraits, perception: &LocalPerception) {
    // Biological decay
    brain.hunger = (brain.hunger + 0.003).min(1.0);
    brain.fatigue = (brain.fatigue + 0.002).min(1.0);

    // Fear: accumulate if danger nearby, otherwise decay
    if perception.danger_detected {
        brain.fear = (brain.fear + 0.05).min(1.0);
    } else {
        brain.fear = (brain.fear - 0.01).max(0.0);
    }

    // Exploration drive fluctuates
    brain.exploration_drive = (brain.exploration_drive + (traits.curiosity - 0.5) * 0.01).clamp(0.0, 1.0);

    // Stress accumulates from high needs
    let need_pressure = brain.hunger + brain.fear + brain.fatigue;
    let chaos_factor = 1.0 - traits.chaos_tolerance;
    brain.stress = (brain.stress + need_pressure * 0.01 * chaos_factor - 0.005).clamp(0.0, 1.0);

    // Agitation
    brain.agitation = (brain.stress * 0.5 + brain.hunger * 0.3 + brain.fear * 0.2).clamp(0.0, 1.0);

    // Social drive
    brain.social_drive = (brain.social_drive + (0.5 - brain.social_drive) * 0.01).clamp(0.0, 1.0);

    // Maintenance drive
    brain.maintenance_drive = (brain.maintenance_drive + 0.001).min(1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_base_ticks() {
        assert_eq!(Action::Move(Direction::N).base_ticks(), 1);
        assert_eq!(Action::Dig(Direction::S).base_ticks(), 7);
        assert_eq!(Action::Rest.base_ticks(), 10);
        assert_eq!(Action::Eat.base_ticks(), 5);
    }

    #[test]
    fn test_needs_increase_over_time() {
        let mut brain = AntBrain::default();
        let traits = AntTraits::random(&mut rand::thread_rng());
        let perception = LocalPerception {
            cells: [[Material::Air; 3]; 3],
            food_detected: false, food_positions: vec![],
            danger_detected: false, danger_positions: vec![],
            nearby_ant_count: 0, nearest_ant_dir: None,
            queen_detected: false, queen_dir: None,
            dirt_adjacent: vec![],
        };

        assert_eq!(brain.hunger, 0.0);
        for _ in 0..100 { update_needs(&mut brain, &traits, &perception); }
        assert!(brain.hunger > 0.2);
        assert!(brain.fatigue > 0.1);
    }

    #[test]
    fn test_select_action_returns_some() {
        let impulses = vec![
            Impulse { action: Action::Idle, weight: 0.1 },
            Impulse { action: Action::Rest, weight: 0.3 },
        ];
        let traits = AntTraits::random(&mut rand::thread_rng());
        let action = select_action(&impulses, &traits);
        // Should pick Rest (higher weight) or deviate (chaos)
        assert!(matches!(action, Action::Rest | Action::Idle | Action::Move(_) | Action::Groom));
    }

    #[test]
    fn test_memory_push_and_check() {
        let mut mem = AntMemory::new(GridPos::new(5, 5));
        mem.push_position(GridPos::new(1, 1));
        mem.push_position(GridPos::new(2, 2));
        assert!(mem.recently_visited(GridPos::new(1, 1), 4));
        assert!(!mem.recently_visited(GridPos::new(3, 3), 4));
    }

    #[test]
    fn test_memory_capacity() {
        let mut mem = AntMemory::new(GridPos::new(5, 5));
        for i in 0..40 {
            mem.push_position(GridPos::new(i as u16, 0));
        }
        assert_eq!(mem.last_positions.len(), 32);
    }
}
```

- [ ] **Step 1: Run tests:** `cargo test -p ant_simulation`
- [ ] **Step 2: Commit:** `git commit -m "feat: add needs update system and tests"`

---

## Phase 2: Integration into simulation

### Task 5: Ant spawning and simulation tick integration

**Files:**
- Modify: `crates/ant_simulation/src/tick.rs` (add ant management)
- Modify: `crates/ant_simulation/src/snapshot.rs` (add ant snapshot data)

Add to `Simulation`:

```rust
pub struct AntState {
    pub bodies: Vec<AntBody>,
    pub brains: Vec<AntBrain>,
    pub memories: Vec<AntMemory>,
    pub traits_vec: Vec<AntTraits>,
}

// In Simulation struct:
pub ants: AntState,

// In Simulation::new:
ants: AntState { bodies: vec![], brains: vec![], memories: vec![], traits_vec: vec![] },

// Spawn initial ants
pub fn spawn_initial_ants(&mut self, count: usize) {
    let home = self.grid.queen_position();
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let pos = GridPos::new(
            (home.x as i32 + rng.gen_range(-3..=3) as i32).clamp(0, self.grid.width as i32 - 1) as u16,
            (home.y as i32 + rng.gen_range(-3..=3) as i32).clamp(0, self.grid.height as i32 - 1) as u16,
        );
        self.ants.bodies.push(AntBody::new(pos));
        self.ants.brains.push(AntBrain::default());
        self.ants.memories.push(AntMemory::new(home));
        self.ants.traits_vec.push(AntTraits::random(&mut rng));
    }
}

// Tick all ants
fn tick_ants(&mut self) -> Vec<AntEvent> {
    let mut events = Vec::new();
    let grid_snapshot = &self.grid; // needed for perception

    for i in 0..self.ants.bodies.len() {
        // Update needs
        let perception = perceive(grid_snapshot, self.ants.bodies[i].pos, self.ants.memories[i].home_position, &[]);
        update_needs(&mut self.ants.brains[i], &self.ants.traits_vec[i], &perception);

        // If mid-action, continue it
        if self.ants.bodies[i].action_ticks > 0 {
            let new_events = execute_action(
                &mut self.ants.bodies[i], &mut self.ants.brains[i],
                &mut self.ants.memories[i], &mut self.grid, self.tick,
            );
            events.extend(new_events);
            continue;
        }

        // Choose new action
        let impulses = calculate_impulses(
            &self.ants.brains[i], &self.ants.memories[i],
            &self.ants.traits_vec[i], &perception, &self.ants.bodies[i],
        );
        let action = select_action(&impulses, &self.ants.traits_vec[i]);
        self.ants.bodies[i].current_action = action;
        self.ants.bodies[i].action_ticks = 0;

        let new_events = execute_action(
            &mut self.ants.bodies[i], &mut self.ants.brains[i],
            &mut self.ants.memories[i], &mut self.grid, self.tick,
        );
        events.extend(new_events);
    }

    events
}
```

Update `Snapshot` to include ant positions for rendering:
```rust
pub struct AntSnapshot {
    pub pos: GridPos,
    pub direction: Direction,
    pub carrying: Option<CarriedItem>,
    pub agitation: f32,
}
// Add to Snapshot: pub ants: Vec<AntSnapshot>
```

- [ ] **Step 1: Run tests:** `cargo test -p ant_simulation`
- [ ] **Step 2: Commit:** `git commit -m "feat: integrate ant spawning and ticking into simulation"`

---

### Task 6: Persistence for ant state

**Files:**
- Modify: `crates/ant_simulation/src/persistence.rs` (save/load ant data)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAnt {
    pub body_pos_x: u16,
    pub body_pos_y: u16,
    pub body_direction: u8,  // discriminant
    pub body_carrying: Option<u8>,
    pub hunger: f32,
    pub fatigue: f32,
    pub fear: f32,
    pub social_drive: f32,
    pub exploration_drive: f32,
    pub maintenance_drive: f32,
    pub stress: f32,
    pub agitation: f32,
    // Memory
    pub last_positions: Vec<(u16, u16)>,
    pub home_x: u16,
    pub home_y: u16,
    // Traits
    pub curiosity: f32,
    pub aggression: f32,
    pub pheromone_sensitivity: f32,
    pub chaos_tolerance: f32,
    pub efficiency: f32,
    pub speed_modifier: f32,
}
```

Add `ants: Vec<SavedAnt>` to `SaveFile`. Update `from_simulation` and `to_simulation`.

- [ ] **Step 1: Run tests:** `cargo test -p ant_simulation` (verify roundtrip with ants)
- [ ] **Step 2: Commit:** `git commit -m "feat: add ant state to persistence save/load"`

---

## Phase 3: Renderer

### Task 7: Ant sprite rendering

**Files:**
- Create: `crates/ant_renderer/src/ants.rs`
- Modify: `crates/ant_renderer/src/sprites.rs` (wire ant spawning)
- Modify: `crates/ant_renderer/src/assets.rs` (ant colors)

In `assets.rs`:
```rust
pub fn ant_body_color(agitation: f32, carrying: Option<ant_simulation::ant::CarriedItem>) -> Color {
    let base = Color::srgb(0.2, 0.15, 0.1); // dark brown-black
    let agitated = Color::srgb(0.35, 0.25, 0.15); // lighter brown
    base.mix(&agitated, agitation)
}

pub fn ant_carrying_tint(item: ant_simulation::ant::CarriedItem) -> Color {
    match item {
        ant_simulation::ant::CarriedItem::Dirt => Color::srgb(0.678, 0.545, 0.235),
        ant_simulation::ant::CarriedItem::Food => Color::srgb(0.91, 0.75, 0.25),
    }
}
```

In `ants.rs`:
```rust
#[derive(Component)]
pub struct AntSprite {
    pub ant_id: usize,
}

pub fn spawn_ant_sprites(
    mut commands: Commands,
    simulation: Res<SimResource>,
) {
    let snap = Snapshot::from_simulation(&simulation);
    for (i, ant) in snap.ants.iter().enumerate() {
        let color = assets::ant_body_color(ant.agitation, ant.carrying);
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CELL_SIZE * 0.8)),
                ..default()
            },
            Transform::from_xyz(
                ant.pos.x as f32 * CELL_SIZE,
                -(ant.pos.y as f32 * CELL_SIZE),
                2.0,
            ),
            AntSprite { ant_id: i },
        ));
    }
}

pub fn update_ant_sprites(
    mut query: Query<(&AntSprite, &mut Sprite, &mut Transform)>,
    simulation_state: Res<SimulationState>,
) {
    // Update ant sprite positions and colors from snapshot
    if let Some(snap) = &simulation_state.snapshot {
        // Match ants by id or re-spawn
    }
}
```

- [ ] **Step 1: Verify compilation:** `cargo check`
- [ ] **Step 2: Commit:** `git commit -m "feat: add ant sprite rendering with color variation"`

---

### Task 8: Wire ant systems into app.rs

**Files:**
- Modify: `crates/ant_renderer/src/app.rs`
- Modify: `crates/ant_renderer/src/sprites.rs`
- Modify: `crates/ant_simulation/src/tick.rs` (call spawn_initial_ants)

Add to `TerrariumPlugin::build()`:
```rust
app.add_systems(Startup, (
    // ... existing
    ants::spawn_ant_sprites,
));

app.add_systems(Update, (
    // ... existing
    ants::update_ant_sprites,
));
```

In `SimResource::load_or_create()`, after creating the simulation:
```rust
if sim.ants.bodies.is_empty() {
    sim.spawn_initial_ants(5);
}
```

- [ ] **Step 1: Run `cargo build` and verify**
- [ ] **Step 2: Run `cargo test -p ant_simulation`**
- [ ] **Step 3: Commit:** `git commit -m "feat: wire ant systems into app — 5 worker ants visible"`

---

## Test Summary

All ant_simulation tests (existing 36 + new ~8):
- Action base_ticks
- Needs increase over time
- Select action returns valid action
- Memory push/check/capacity
- Roundtrip persistence with ant state
- Perception in empty grid
- Collision avoidance
- Execute action produces events
