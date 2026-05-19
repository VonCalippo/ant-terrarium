use crate::grid::{Direction, Grid, GridPos, Material, PheromoneType};

pub struct Queen {
    pub pos: GridPos,
    pub health: f32,
    pub stress: f32,
    pub hunger: f32,
    pub egg_progress: f32,
    pub food_reserve: u16,
    pub alive: bool,
}

impl Queen {
    pub fn new(pos: GridPos) -> Self {
        Self {
            pos,
            health: 1.0,
            stress: 0.0,
            hunger: 0.0,
            egg_progress: 0.0,
            food_reserve: 0,
            alive: true,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> Vec<QueenEvent> {
        if !self.alive { return vec![]; }

        let mut events = Vec::new();

        self.hunger = (self.hunger + 0.0005).min(1.0);
        self.stress = (self.stress + 0.0005).min(1.0);

        // Auto-feed only when there's surplus (keep reserve for eggs)
        if self.hunger > 0.5 && self.food_reserve > 10 {
            self.food_reserve -= 1;
            self.hunger = (self.hunger - 0.4).max(0.0);
        }

        // Queen movement: rarely moves, only under stress or danger
        let danger_nearby = grid.pheromone_at(self.pos, PheromoneType::Danger) > 50
            || grid.pheromone_at(self.pos, PheromoneType::Death) > 50;
        let should_move = danger_nearby || self.stress > 0.8 || self.hunger > 0.9;

        if should_move && rand::thread_rng().gen_range(0..3) == 0 {
            // Find a safe adjacent cell to move to
            if let Some(new_pos) = find_safe_spot(grid, self.pos) {
                self.pos = new_pos;
                self.stress = (self.stress - 0.2).max(0.0);
            }
        }

        // Emit QUEEN pheromone and HOME pheromone
        grid.deposit_pheromone(self.pos, PheromoneType::Queen, 20);
        grid.deposit_pheromone(self.pos, PheromoneType::Home, 40);

        // Health
        if self.hunger > 0.9 || self.stress > 0.9 {
            self.health = (self.health - 0.001).max(0.0);
        } else if self.hunger < 0.3 && self.stress < 0.3 {
            self.health = (self.health + 0.0005).min(1.0);
        }

        // Egg laying
        if self.food_reserve >= 5 && self.hunger < 0.7 && self.stress < 0.8 && self.health > 0.1 {
            self.egg_progress += 0.05; // ~20 ticks per egg when well-fed
            if self.egg_progress >= 1.0 {
                self.egg_progress = 0.0;
                self.food_reserve = self.food_reserve.saturating_sub(5);
                if let Some(egg_pos) = find_adjacent_air(grid, self.pos) {
                    grid.set_material(egg_pos, Material::Egg);
                    events.push(QueenEvent::LaidEgg { pos: egg_pos });
                }
            }
        }

        // Death
        if self.health <= 0.0 {
            self.alive = false;
            grid.deposit_pheromone(self.pos, PheromoneType::Death, 200);
            events.push(QueenEvent::Died { pos: self.pos });
        }

        events
    }

    pub fn deliver_food(&mut self) {
        self.food_reserve = self.food_reserve.saturating_add(1);
        self.hunger = (self.hunger - 0.3).max(0.0);
        self.stress = (self.stress - 0.05).max(0.0);
    }

    pub fn reduce_stress(&mut self, amount: f32) {
        self.stress = (self.stress - amount).max(0.0);
    }
}

fn find_safe_spot(grid: &Grid, pos: GridPos) -> Option<GridPos> {
    let mut rng = rand::thread_rng();
    let mut dirs: Vec<Direction> = Vec::from(Direction::ALL);
    for i in (1..dirs.len()).rev() { let j = rng.gen_range(0..=i); dirs.swap(i, j); }
    for dir in &dirs {
        if let Some(np) = pos.neighbor(*dir) {
            if let Some(cell) = grid.get(np) {
                // Prefer air cells with low danger
                if cell.material == Material::Air
                    && cell.pheromones.danger < 30
                    && cell.pheromones.death < 30
                {
                    return Some(np);
                }
            }
        }
    }
    None
}

fn find_adjacent_air(grid: &Grid, pos: GridPos) -> Option<GridPos> {
    use crate::grid::Direction;
    let mut rng = rand::thread_rng();
    let mut dirs: Vec<Direction> = Vec::from(Direction::ALL);
    // Shuffle for randomness
    for i in (1..dirs.len()).rev() {
        let j = rng.gen_range(0..=i);
        dirs.swap(i, j);
    }
    for dir in &dirs {
        if let Some(np) = pos.neighbor(*dir) {
            if let Some(cell) = grid.get(np) {
                if cell.material == Material::Air {
                    return Some(np);
                }
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
pub enum QueenEvent {
    LaidEgg { pos: GridPos },
    Died { pos: GridPos },
}

// ── Life cycle stage tracker ──

#[derive(Debug, Clone)]
pub struct LifeStage {
    pub pos: GridPos,
    pub stage: Stage,
    pub progress: u32,      // ticks in current stage
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Egg,
    Larva,
    Pupa,
}

impl Stage {
    pub fn duration(self) -> u32 {
        match self {
            Stage::Egg => 600,
            Stage::Larva => 600,
            Stage::Pupa => 600,
        }
    }

    pub fn material(self) -> Material {
        match self {
            Stage::Egg => Material::Egg,
            Stage::Larva => Material::Larva,
            Stage::Pupa => Material::Larva, // reuse larva sprite
        }
    }

    pub fn next(self) -> Option<Stage> {
        match self {
            Stage::Egg => Some(Stage::Larva),
            Stage::Larva => Some(Stage::Pupa),
            Stage::Pupa => None, // becomes adult
        }
    }
}

pub fn tick_life_stages(stages: &mut Vec<LifeStage>, grid: &mut Grid) -> Vec<LifeEvent> {
    let mut events = Vec::new();
    let mut completed = Vec::new();
    let mut to_add: Vec<LifeStage> = Vec::new();

    for (i, stage) in stages.iter_mut().enumerate() {
        // Check if the cell still holds the stage material (might have been overwritten)
        if let Some(cell) = grid.get(stage.pos) {
            if cell.material != stage.stage.material() {
                // Stage was destroyed; consider it dead
                completed.push(i);
                continue;
            }
        }

        stage.progress += 1;

        if stage.progress >= stage.stage.duration() {
            if let Some(next_stage) = stage.stage.next() {
                grid.set_material(stage.pos, next_stage.material());
                events.push(LifeEvent::Advanced {
                    pos: stage.pos,
                    from: stage.stage,
                    to: next_stage,
                });
                to_add.push(LifeStage { pos: stage.pos, stage: next_stage, progress: 0 });
            } else {
                // Pupa → Adult
                grid.set_material(stage.pos, Material::Air);
                events.push(LifeEvent::Hatched { pos: stage.pos });
            }
            completed.push(i);
        }
    }

    // Apply changes
    for i in completed.into_iter().rev() {
        stages.remove(i);
    }
    stages.extend(to_add);

    events
}

#[derive(Debug, Clone)]
pub enum LifeEvent {
    Advanced { pos: GridPos, from: Stage, to: Stage },
    Hatched { pos: GridPos },
}

// ── Ant Aging ──

#[derive(Debug, Clone)]
pub struct AntAge {
    pub age: u32,
    pub max_age: u32,
    pub health: f32,
}

impl AntAge {
    pub fn new(rng: &mut impl rand::Rng) -> Self {
        Self {
            age: rng.gen_range(0..2000),
            max_age: rng.gen_range(50000..80000),
            health: 1.0,
        }
    }

    pub fn adult(max_age_offset: u32, rng: &mut impl rand::Rng) -> Self {
        Self {
            age: 0,
            max_age: 50000 + max_age_offset + rng.gen_range(0..30000),
            health: 1.0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.age += 1;
        if self.age > self.max_age.saturating_mul(4) / 5 {
            self.health = (self.health - 0.0001).max(0.0);
        }
        self.health <= 0.0
    }
}

// ── Corpse decay ──

pub fn tick_corpses(grid: &mut Grid) {
    // Process organic waste: emit death pheromone continuously
    let width = grid.width;
    let height = grid.height;

    for y in 0..height {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            if grid.get(pos).map(|c| c.material == Material::OrganicWaste).unwrap_or(false) {
                grid.deposit_pheromone(pos, PheromoneType::Death, 5);
            }
        }
    }
}

use rand::Rng;
