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

    pub fn from_grid(grid: Grid) -> Self {
        Self {
            grid,
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

    #[test]
    fn test_simulation_from_grid() {
        let grid = Grid::generate_initial_world(10, 10);
        let sim = Simulation::from_grid(grid);
        assert_eq!(sim.tick, 0);
        assert_eq!(sim.grid.width, 10);
    }
}
