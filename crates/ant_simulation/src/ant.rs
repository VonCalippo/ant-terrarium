use rand::Rng;
use crate::grid::{Grid, GridPos, Direction, Material, PheromoneLayer, PheromoneType};

// ── Actions ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Action::Move(_) => 0,
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

// ── Carried Item ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarriedItem {
    Dirt,
    Food,
}

// ── Ant Components ──

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

    pub fn action_speed_modifier(&self, traits: Option<&AntTraits>) -> f32 {
        traits.map(|t| t.speed_modifier).unwrap_or(1.0)
    }

    pub fn ticks_for_action(&self, traits: Option<&AntTraits>) -> u8 {
        let base = self.current_action.base_ticks() as f32;
        let modif = self.action_speed_modifier(traits);
        (base / modif).max(1.0) as u8
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
    pub last_positions: Vec<GridPos>,
    pub nest_direction: Option<Direction>,
    pub recent_food: Vec<GridPos>,
    pub recent_dangers: Vec<GridPos>,
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
        let start = self.last_positions.len().saturating_sub(n);
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
    pub fn random(rng: &mut impl Rng) -> Self {
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

fn random_trait(rng: &mut impl Rng) -> f32 {
    (rng.gen::<f32>() * 0.3 + 0.35).clamp(0.0, 1.0)
}

// ── Impulse ──

#[derive(Debug, Clone)]
pub struct Impulse {
    pub action: Action,
    pub weight: f32,
}

// ── Local Perception ──

pub struct LocalPerception {
    pub cells: [[Material; 3]; 3],
    pub pheromones: [[PheromoneLayer; 3]; 3],
    pub food_detected: bool,
    pub food_positions: Vec<(i8, i8)>,
    pub danger_detected: bool,
    pub danger_positions: Vec<(i8, i8)>,
    pub nearby_ant_count: u8,
    pub nearest_ant_dir: Option<Direction>,
    pub queen_detected: bool,
    pub queen_dir: Option<Direction>,
    pub dirt_adjacent: Vec<Direction>,
}

impl LocalPerception {
    pub fn strongest_pheromone_dir(&self, ptype: PheromoneType) -> Option<(Direction, u8)> {
        let get = |l: &PheromoneLayer| -> u8 {
            match ptype {
                PheromoneType::Food => l.food,
                PheromoneType::Home => l.home,
                PheromoneType::Danger => l.danger,
                PheromoneType::Dig => l.dig,
                PheromoneType::Queen => l.queen,
                PheromoneType::Death => l.death,
                PheromoneType::Waste => l.waste,
            }
        };
        let center_val = get(&self.pheromones[1][1]);
        let mut best_dir: Option<(Direction, u8)> = None;
        let dirs = [
            (Direction::N, 1, 0), (Direction::S, 1, 2),
            (Direction::E, 2, 1), (Direction::W, 0, 1),
            (Direction::NE, 2, 0), (Direction::NW, 0, 0),
            (Direction::SE, 2, 2), (Direction::SW, 0, 2),
        ];
        for (dir, sx, sy) in &dirs {
            let val = get(&self.pheromones[*sy][*sx]);
            if val > 0 && val > center_val {
                match best_dir {
                    Some((_, best_val)) if val > best_val => best_dir = Some((*dir, val)),
                    None => best_dir = Some((*dir, val)),
                    _ => {}
                }
            }
        }
        best_dir
    }
}

pub fn perceive(
    grid: &Grid,
    pos: GridPos,
    home: GridPos,
) -> LocalPerception {
    let mut cells = [[Material::Air; 3]; 3];
    let mut pheromones = [[PheromoneLayer::default(); 3]; 3];
    let mut food_positions = Vec::new();
    let danger_positions = Vec::new();
    let mut food_detected = false;
    let danger_detected = false;
    let mut dirt_adjacent = Vec::new();
    let nearby_ant_count: u8 = 0;
    let mut queen_detected = false;
    let mut queen_dir = None;

    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            let sx = (dx + 1) as usize;
            let sy = (dy + 1) as usize;
            let nx = pos.x as i32 + dx as i32;
            let ny = pos.y as i32 + dy as i32;
            if nx < 0 || ny < 0 {
                pheromones[sy][sx] = PheromoneLayer::default();
                continue;
            }
            let np = GridPos::new(nx as u16, ny as u16);
            if let Some(cell) = grid.get(np) {
                pheromones[sy][sx] = cell.pheromones;
                cells[sy][sx] = cell.material;

                if dx == 0 && dy == 0 { continue; }

                match cell.material {
                    Material::Food | Material::Fungus => {
                        food_detected = true;
                        food_positions.push((dx, dy));
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
            } else {
                pheromones[sy][sx] = PheromoneLayer::default();
            }
        }
    }

    let dx = home.x as i32 - pos.x as i32;
    let dy = home.y as i32 - pos.y as i32;
    let dist = ((dx * dx + dy * dy) as f32).sqrt();
    if dist <= 6.0 && dist > 0.0 {
        queen_detected = true;
        queen_dir = Some(approx_direction(dx, dy));
    }

    LocalPerception {
        cells, pheromones,
        food_detected, food_positions,
        danger_detected, danger_positions,
        nearby_ant_count, nearest_ant_dir: None,
        queen_detected, queen_dir,
        dirt_adjacent,
    }
}

// ── Decision Making ──

pub fn calculate_impulses(
    brain: &AntBrain,
    _memory: &AntMemory,
    traits: &AntTraits,
    perception: &LocalPerception,
    body: &AntBody,
) -> Vec<Impulse> {
    let mut impulses = Vec::new();

    if perception.food_detected && brain.hunger > 0.3 {
        let w = if brain.hunger > 0.6 { 0.8 } else { 0.4 };
        impulses.push(Impulse { action: Action::CollectFood, weight: w });
    }
    if brain.hunger > 0.8 && !perception.food_detected {
        let dir = random_direction();
        impulses.push(Impulse { action: Action::Move(dir), weight: 0.7 });
    }
    if body.carrying == Some(CarriedItem::Food) && perception.queen_detected {
        impulses.push(Impulse {
            action: Action::CarryFood { to: perception.queen_dir.map(|d| body.pos.neighbor(d).unwrap_or(body.pos)).unwrap_or(body.pos) },
            weight: 0.9,
        });
    }
    if body.carrying == Some(CarriedItem::Food) && !perception.queen_detected {
        if let Some(dir) = perception.queen_dir {
            impulses.push(Impulse { action: Action::Move(dir), weight: 0.4 });
        }
    }

    if brain.fatigue > 0.5 && !perception.danger_detected {
        impulses.push(Impulse { action: Action::Rest, weight: brain.fatigue.min(0.8) });
    }

    if perception.danger_detected && brain.fear > 0.3 {
        if perception.danger_positions.first().is_some() {
            let w = 0.5 + (1.0 - traits.aggression) * 0.4;
            impulses.push(Impulse { action: Action::Flee { from: body.pos }, weight: w });
        }
    }

    if brain.exploration_drive > 0.6 && matches!(body.current_action, Action::Idle) {
        let w = traits.curiosity * 0.6;
        impulses.push(Impulse { action: Action::Move(random_direction()), weight: w });
    }

    if brain.maintenance_drive > 0.5 && !perception.dirt_adjacent.is_empty() {
        let dir = perception.dirt_adjacent[0];
        impulses.push(Impulse { action: Action::Dig(dir), weight: 0.2 + traits.efficiency * 0.5 });
    }

    if body.carrying == Some(CarriedItem::Dirt) {
        impulses.push(Impulse { action: Action::CarryDirt { to: body.pos }, weight: 0.6 });
    }

    if brain.stress > 0.7 {
        impulses.push(Impulse { action: Action::Groom, weight: 0.5 });
    }

    // ── Pheromone-driven impulses ──
    let ps = traits.pheromone_sensitivity;

    // FOOD pheromone attraction
    if let Some((dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Food) {
        let w = (strength as f32 / 255.0) * 0.6 * (1.0 + ps);
        impulses.push(Impulse { action: Action::Move(dir), weight: w });
    }

    // HOME pheromone attraction
    if let Some((dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Home) {
        let w = (strength as f32 / 255.0) * 0.5 * (1.0 + ps);
        impulses.push(Impulse { action: Action::Move(dir), weight: w });
    }

    // DANGER pheromone repel
    if let Some((_dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Danger) {
        let w = (strength as f32 / 255.0) * 0.7 * (1.0 + ps);
        impulses.push(Impulse { action: Action::Flee { from: body.pos }, weight: w });
    }

    // DIG pheromone attraction
    if let Some((dir, strength)) = perception.strongest_pheromone_dir(PheromoneType::Dig) {
        let w = (strength as f32 / 255.0) * 0.3 * (1.0 + ps);
        impulses.push(Impulse { action: Action::Dig(dir), weight: w });
    }

    // DEATH pheromone repel
    let d_strength = perception.pheromones[1][1].death as f32 / 255.0;
    if d_strength > 0.0 {
        let w = d_strength * 0.6;
        impulses.push(Impulse { action: Action::Move(opposite_dir(body.direction)), weight: w });
    }

    impulses.push(Impulse { action: Action::Idle, weight: 0.1 });
    impulses
}

pub fn select_action(impulses: &[Impulse], traits: &AntTraits) -> Action {
    let mut rng = rand::thread_rng();

    // Chaos deviation chance
    if rng.gen::<f32>() < traits.chaos_tolerance * 0.1 {
        return match rng.gen_range(0..4) {
            0 => Action::Idle,
            1 => Action::Move(random_direction()),
            2 => Action::Rest,
            _ => Action::Groom,
        };
    }

    impulses.iter()
        .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap())
        .map(|i| i.action)
        .unwrap_or(Action::Idle)
}

// ── Movement & Execution ──

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

pub fn execute_action(
    body: &mut AntBody,
    brain: &mut AntBrain,
    memory: &mut AntMemory,
    grid: &mut Grid,
    traits: Option<&AntTraits>,
) -> Vec<AntEvent> {
    let mut events = Vec::new();
    let ticks_needed = body.ticks_for_action(traits);

    body.action_ticks += 1;
    if body.action_ticks < ticks_needed {
        return events;
    }
    body.action_ticks = 0;

    match body.current_action {
        Action::Move(dir) => {
            if let Some(new_pos) = body.pos.neighbor(dir) {
                if grid.contains(new_pos) {
                    if let Some(cell) = grid.get(new_pos) {
                        if cell.material.is_walkable() && memory.recently_visited(new_pos, 2) {
                            let alt = alternate_direction(dir);
                            if let Some(alt_pos) = body.pos.neighbor(alt) {
                                if grid.contains(alt_pos) && grid.get(alt_pos).map(|c| c.material.is_walkable()).unwrap_or(false) && !memory.recently_visited(alt_pos, 2) {
                                    body.pos = alt_pos;
                                    body.direction = alt;
                                    memory.push_position(alt_pos);
                                    // Deposit FOOD trail if carrying food
                                    if body.carrying == Some(CarriedItem::Food) {
                                        grid.deposit_pheromone(alt_pos, PheromoneType::Food, 30);
                                    }
                                    events.push(AntEvent::Moved { from: body.pos, to: alt_pos });
                                    return events;
                                }
                            }
                            events.push(AntEvent::Blocked { pos: body.pos });
                        } else {
                            body.pos = new_pos;
                            body.direction = dir;
                            memory.push_position(new_pos);
                            if body.carrying == Some(CarriedItem::Food) {
                                grid.deposit_pheromone(new_pos, PheromoneType::Food, 30);
                            }
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
                        crate::terrain::start_dig(grid, target_pos, &mut Vec::new());
                        grid.deposit_pheromone(target_pos, PheromoneType::Dig, 100);
                        events.push(AntEvent::StartedDigging { pos: target_pos });
                    }
                }
            }
        }
        Action::CollectFood => {
            for dir in &Direction::ALL {
                if let Some(pos) = body.pos.neighbor(*dir) {
                    let is_food = grid.get(pos).map(|c| c.material == Material::Food).unwrap_or(false);
                    let is_fungus = grid.get(pos).map(|c| c.material == Material::Fungus).unwrap_or(false);
                    if is_food || is_fungus {
                        grid.set_material(pos, Material::Air);
                        body.carrying = Some(CarriedItem::Food);
                        brain.hunger = if is_fungus {
                            (brain.hunger - 0.2).max(0.0)
                        } else {
                            (brain.hunger - 0.3).max(0.0)
                        };
                        memory.recent_food.push(pos);
                        if memory.recent_food.len() > 8 { memory.recent_food.remove(0); }
                        events.push(AntEvent::CollectedFood { pos });
                        break;
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
            for dir in &Direction::ALL {
                if let Some(pos) = body.pos.neighbor(*dir) {
                    if let Some(cell) = grid.get(pos) {
                        if cell.material == Material::Air {
                            grid.set_material(pos, Material::LooseDirt);
                            grid.deposit_pheromone(pos, PheromoneType::Waste, 60);
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
            grid.deposit_pheromone(from, PheromoneType::Danger, 150);
            if let Some(new_pos) = body.pos.neighbor(dir) {
                if grid.contains(new_pos) && grid.get(new_pos).map(|c| c.material.is_walkable()).unwrap_or(false) {
                    body.pos = new_pos;
                    memory.push_position(new_pos);
                    memory.recent_dangers.push(from);
                    if memory.recent_dangers.len() > 4 { memory.recent_dangers.remove(0); }
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

// ── Needs Update ──

pub fn update_needs(brain: &mut AntBrain, traits: &AntTraits, perception: &LocalPerception) {
    brain.hunger = (brain.hunger + 0.003).min(1.0);
    brain.fatigue = (brain.fatigue + 0.002).min(1.0);

    if perception.danger_detected {
        brain.fear = (brain.fear + 0.05).min(1.0);
    } else {
        brain.fear = (brain.fear - 0.01).max(0.0);
    }

    brain.exploration_drive = (brain.exploration_drive + (traits.curiosity - 0.5) * 0.01).clamp(0.0, 1.0);

    let need_pressure = brain.hunger + brain.fear + brain.fatigue;
    let chaos_factor = 1.0 - traits.chaos_tolerance;
    brain.stress = (brain.stress + need_pressure * 0.01 * chaos_factor - 0.005).clamp(0.0, 1.0);

    brain.agitation = (brain.stress * 0.5 + brain.hunger * 0.3 + brain.fear * 0.2).clamp(0.0, 1.0);
    brain.social_drive = (brain.social_drive + (0.5 - brain.social_drive) * 0.01).clamp(0.0, 1.0);
    brain.maintenance_drive = (brain.maintenance_drive + 0.001).min(1.0);

    // Pheromone effects
    let q_strength = perception.pheromones[1][1].queen as f32 / 255.0;
    if q_strength > 0.0 {
        brain.stress = (brain.stress - q_strength * 0.02).max(0.0);
    }

    let danger_phero = perception.pheromones[1][1].danger as f32 / 255.0;
    if danger_phero > 0.0 {
        brain.fear = (brain.fear + danger_phero * 0.005).min(1.0);
    }

    let death_phero = perception.pheromones[1][1].death as f32 / 255.0;
    if death_phero > 0.0 {
        brain.fear = (brain.fear + death_phero * 0.008).min(1.0);
    }
}

// ── Helpers ──

fn approx_direction(dx: i32, dy: i32) -> Direction {
    if dx.abs() > dy.abs() * 2 { return if dx > 0 { Direction::E } else { Direction::W }; }
    if dy.abs() > dx.abs() * 2 { return if dy > 0 { Direction::S } else { Direction::N }; }
    match (dx > 0, dy > 0) {
        (true, true) => Direction::SE, (true, false) => Direction::NE,
        (false, true) => Direction::SW, (false, false) => Direction::NW,
    }
}

fn random_direction() -> Direction {
    Direction::ALL[rand::thread_rng().gen_range(0..8)]
}

fn alternate_direction(dir: Direction) -> Direction {
    use Direction::*;
    match dir {
        N => NE, S => SW, E => SE, W => NW,
        NE => E, NW => N, SE => S, SW => W,
    }
}

fn opposite_dir(dir: Direction) -> Direction {
    use Direction::*;
    match dir {
        N => S, S => N, E => W, W => E,
        NE => SW, SW => NE, NW => SE, SE => NW,
    }
}

// ── Ant State container ──

#[derive(Debug, Clone, Default)]
pub struct AntState {
    pub bodies: Vec<AntBody>,
    pub brains: Vec<AntBrain>,
    pub memories: Vec<AntMemory>,
    pub traits_vec: Vec<AntTraits>,
}

impl AntState {
    pub fn len(&self) -> usize { self.bodies.len() }

    pub fn spawn(&mut self, pos: GridPos, home: GridPos, rng: &mut impl Rng) {
        self.bodies.push(AntBody::new(pos));
        self.brains.push(AntBrain::default());
        self.memories.push(AntMemory::new(home));
        self.traits_vec.push(AntTraits::random(rng));
    }

    pub fn spawn_initial_ants(&mut self, count: usize, grid: &Grid) {
        let home = grid.queen_position();
        let surface_y = grid.surface_y();
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let x = (home.x as i32 + rng.gen_range(-4..=4) as i32).clamp(0, grid.width as i32 - 1) as u16;
            // Spawn on air cell just above surface so ant can walk on dirt
            let y = if surface_y > 0 { surface_y - 1 } else { 0 };
            self.spawn(GridPos::new(x, y), home, &mut rng);
        }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_base_ticks() {
        assert_eq!(Action::Move(Direction::N).base_ticks(), 0);
        assert_eq!(Action::Dig(Direction::S).base_ticks(), 7);
        assert_eq!(Action::Rest.base_ticks(), 10);
        assert_eq!(Action::Eat.base_ticks(), 5);
        assert_eq!(Action::Idle.base_ticks(), 1);
    }

    #[test]
    fn test_needs_increase_over_time() {
        let mut brain = AntBrain::default();
        let traits = AntTraits::random(&mut rand::thread_rng());
        let perception = LocalPerception {
            cells: [[Material::Air; 3]; 3],
            pheromones: [[PheromoneLayer::default(); 3]; 3],
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

    #[test]
    fn test_ant_state_spawn() {
        let grid = Grid::generate_initial_world(128, 96);
        let mut state = AntState::default();
        state.spawn_initial_ants(5, &grid);
        assert_eq!(state.len(), 5);
        assert_eq!(state.bodies.len(), 5);
        assert_eq!(state.brains.len(), 5);
        assert_eq!(state.memories.len(), 5);
        assert_eq!(state.traits_vec.len(), 5);
    }

    #[test]
    fn test_perceive_sees_air() {
        let grid = Grid::new(10, 10);
        let perception = perceive(&grid, GridPos::new(5, 5), GridPos::new(5, 0));
        assert!(!perception.food_detected);
        assert!(!perception.danger_detected);
    }
}
