use crate::ant::{AntState, AntEvent, ColonySignal, perceive, update_needs, calculate_impulses, select_action, execute_action};
use crate::ecology::tick_ecology;
use crate::grid::Grid;
use crate::queen::{AntAge, Queen, QueenEvent, LifeStage, tick_life_stages, tick_corpses};
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
            Speed::Normal => Some(50),   // ~20 ticks/sec
            Speed::Fast => Some(25),      // ~40 ticks/sec
            Speed::Fastest => Some(10),   // ~100 ticks/sec
        }
    }

    pub fn ticks_per_second(self) -> u16 {
        match self {
            Speed::Paused => 0,
            Speed::Normal => 20,
            Speed::Fast => 40,
            Speed::Fastest => 100,
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
    pub ants: AntState,
    pub ant_ages: Vec<AntAge>,
    pub ant_events: Vec<AntEvent>,
    pub queen: Queen,
    pub queen_events: Vec<QueenEvent>,
    pub life_stages: Vec<LifeStage>,
}

impl Simulation {
    pub fn new(width: u16, height: u16) -> Self {
        let grid = Grid::new(width, height);
        let queen_pos = grid.queen_position();
        Self {
            grid,
            tick: 0,
            speed: Speed::Normal,
            events: Vec::new(),
            pending_digs: Vec::new(),
            ants: AntState::default(),
            ant_ages: Vec::new(),
            ant_events: Vec::new(),
            queen: Queen::new(queen_pos),
            queen_events: Vec::new(),
            life_stages: Vec::new(),
        }
    }

    pub fn from_grid(grid: Grid) -> Self {
        let queen_pos = grid.queen_position();
        Self {
            grid,
            tick: 0,
            speed: Speed::Normal,
            events: Vec::new(),
            pending_digs: Vec::new(),
            ants: AntState::default(),
            ant_ages: Vec::new(),
            ant_events: Vec::new(),
            queen: Queen::new(queen_pos),
            queen_events: Vec::new(),
            life_stages: Vec::new(),
        }
    }

    pub fn spawn_initial_ants(&mut self, count: usize) {
        self.ants.spawn_initial_ants(count, &self.grid);
        for _ in 0..count {
            self.ant_ages.push(AntAge::new(&mut rand::thread_rng()));
        }
    }

    pub fn compute_colony_signal(&self) -> ColonySignal {
        let ant_count = self.ants.bodies.len().max(1) as f32;

        // Food scarcity: how many ants are very hungry
        let starving = self.ants.brains.iter().filter(|b| b.hunger > 0.7).count() as f32;
        let food_scarcity = (starving / ant_count).clamp(0.0, 1.0);

        // Crowding: ants per air cell in the nest area (rough: ants / 10)
        let crowding = (ant_count / 10.0).clamp(0.0, 1.0);

        // Queen distress
        let queen_distress = if self.queen.alive {
            (self.queen.hunger * 0.5 + self.queen.stress * 0.5).clamp(0.0, 1.0)
        } else {
            1.0
        };

        ColonySignal { food_scarcity, crowding, queen_distress }
    }

    pub fn tick_ants(&mut self) -> Vec<AntEvent> {
        let mut events = Vec::new();
        let signal = self.compute_colony_signal();

        for i in 0..self.ants.bodies.len() {
            let perception = perceive(&self.grid, self.ants.bodies[i].pos, self.ants.memories[i].home_position);
            update_needs(&mut self.ants.brains[i], &self.ants.traits_vec[i], &perception);

            // Dynamic role re-evaluation (every ~100 ticks)
            if self.tick % 100 == 0 {
                let signal = self.compute_colony_signal();
                self.ants.bodies[i].role = self.ants.bodies[i].role.next(&signal);
            }

            // Interrupt current action if critical need
            if self.ants.bodies[i].action_ticks > 0
                && (self.ants.brains[i].hunger > 0.85 || self.ants.brains[i].fear > 0.7)
            {
                self.ants.bodies[i].action_ticks = 0;
                self.ants.bodies[i].current_action = crate::ant::Action::Idle;
            }

            let new_events = execute_action(
                &mut self.ants.bodies[i],
                &mut self.ants.brains[i],
                &mut self.ants.memories[i],
                &mut self.grid,
                Some(&self.ants.traits_vec[i]),
                &mut self.pending_digs,
            );
            events.extend(new_events);

            if self.ants.bodies[i].action_ticks > 0 {
                continue;
            }

            // Choose new action
            let impulses = calculate_impulses(
                &self.ants.brains[i],
                &self.ants.memories[i],
                &self.ants.traits_vec[i],
                &perception,
                &self.ants.bodies[i],
                &signal,
            );
            let action = select_action(&impulses, &self.ants.traits_vec[i]);
            self.ants.bodies[i].current_action = action;

            let followup = execute_action(
                &mut self.ants.bodies[i],
                &mut self.ants.brains[i],
                &mut self.ants.memories[i],
                &mut self.grid,
                Some(&self.ants.traits_vec[i]),
                &mut self.pending_digs,
            );
            events.extend(followup);
        }

        // Process delivery events: queens receive food
        for event in &events {
            if let AntEvent::DeliveredFood { pos } = event {
                // If delivery happened at queen location, feed the queen
                if *pos == self.queen.pos {
                    self.queen.deliver_food();
                }
            }
        }

        events
    }

    pub fn tick(&mut self) -> TickResult {
        self.tick += 1;
        self.events.clear();
        self.ant_events.clear();
        self.queen_events.clear();

        self.grid.evaporate_pheromones();
        // Ecology is expensive — run every 10 ticks
        if self.tick % 10 == 0 {
            tick_ecology(&mut self.grid);
        }

        // Queen tick
        self.queen_events = self.queen.tick(&mut self.grid);

        // Life stages
        let life_events = tick_life_stages(&mut self.life_stages, &mut self.grid);
        for event in &life_events {
            if let crate::queen::LifeEvent::Hatched { pos } = event {
                // Spawn new adult ant
                self.ants.spawn(*pos, self.queen.pos, &mut rand::thread_rng());
                self.ant_ages.push(AntAge::adult(0, &mut rand::thread_rng()));
            }
        }

        // Corpses
        tick_corpses(&mut self.grid);

        // Ant aging & death
        let mut deaths = Vec::new();
        for i in 0..self.ant_ages.len() {
            if self.ant_ages[i].tick() {
                deaths.push(i);
            }
        }

        // Process ant deaths
        for i in deaths.into_iter().rev() {
            if i < self.ants.bodies.len() {
                let pos = self.ants.bodies[i].pos;
                self.grid.set_material(pos, crate::grid::Material::OrganicWaste);
                self.grid.deposit_pheromone(pos, crate::grid::PheromoneType::Death, 200);
                self.ants.bodies.remove(i);
                self.ants.brains.remove(i);
                self.ants.memories.remove(i);
                self.ants.traits_vec.remove(i);
                self.ant_ages.remove(i);
            }
        }

        let dig_events = process_digging(&mut self.grid, &mut self.pending_digs);
        let stability_events = update_stability(&mut self.grid);
        let ant_events = self.tick_ants();

        // Worker feeding queen: check if any ant delivered food near queen
        for event in &ant_events {
            if let AntEvent::DeliveredFood { pos } = event {
                let dx = pos.x as i32 - self.queen.pos.x as i32;
                let dy = pos.y as i32 - self.queen.pos.y as i32;
                if dx.abs() <= 2 && dy.abs() <= 2 {
                    self.queen.deliver_food();
                }
            }
        }

        self.events.extend(dig_events);
        self.events.extend(stability_events);
        self.ant_events = ant_events;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_tick_interval() {
        assert_eq!(Speed::Paused.tick_interval_ms(), None);
        assert_eq!(Speed::Normal.tick_interval_ms(), Some(50));
        assert_eq!(Speed::Fast.tick_interval_ms(), Some(25));
        assert_eq!(Speed::Fastest.tick_interval_ms(), Some(10));
    }

    #[test]
    fn test_speed_ticks_per_second() {
        assert_eq!(Speed::Paused.ticks_per_second(), 0);
        assert_eq!(Speed::Normal.ticks_per_second(), 20);
        assert_eq!(Speed::Fast.ticks_per_second(), 40);
        assert_eq!(Speed::Fastest.ticks_per_second(), 100);
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
        assert_eq!(sim.tick_interval_ms(), Some(25));
    }

    #[test]
    fn test_paused_no_ticks() {
        let mut sim = Simulation::new(10, 10);
        sim.set_speed(Speed::Paused);
        assert_eq!(sim.speed.ticks_per_second(), 0);
        assert_eq!(sim.tick_interval_ms(), None);
    }

    #[test]
    fn test_simulation_from_grid() {
        let grid = Grid::generate_initial_world(10, 10);
        let sim = Simulation::from_grid(grid);
        assert_eq!(sim.tick, 0);
        assert_eq!(sim.grid.width, 10);
    }
}
