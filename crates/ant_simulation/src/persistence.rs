use serde::{Serialize, Deserialize};
use crate::grid::{Cell, GridPos, Material, PheromoneLayer};
use crate::tick::Simulation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<SavedCell>,
    pub ants: Vec<SavedAnt>,
    pub rng_state: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCell {
    pub material: Material,
    pub humidity: u8,
    pub temperature: i16,
    pub stability: u8,
    pub pheromones: PheromoneLayer,
    pub organic_matter: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAnt {
    pub pos_x: u16,
    pub pos_y: u16,
    pub home_x: u16,
    pub home_y: u16,
    pub hunger: f32,
    pub fatigue: f32,
    pub fear: f32,
    pub social_drive: f32,
    pub exploration_drive: f32,
    pub maintenance_drive: f32,
    pub stress: f32,
    pub agitation: f32,
    pub curiosity: f32,
    pub aggression: f32,
    pub pheromone_sensitivity: f32,
    pub chaos_tolerance: f32,
    pub efficiency: f32,
    pub speed_modifier: f32,
}

impl SaveFile {
    pub fn from_simulation(sim: &Simulation) -> Self {
        let cells: Vec<SavedCell> = sim.grid.cells.iter().map(|c| SavedCell {
            material: c.material,
            humidity: c.humidity,
            temperature: c.temperature,
            stability: c.stability,
            pheromones: c.pheromones,
            organic_matter: c.organic_matter,
        }).collect();

        let ants: Vec<SavedAnt> = sim.ants.bodies.iter().enumerate().map(|(i, body)| {
            let brain = &sim.ants.brains[i];
            let mem = &sim.ants.memories[i];
            let traits = &sim.ants.traits_vec[i];
            SavedAnt {
                pos_x: body.pos.x, pos_y: body.pos.y,
                home_x: mem.home_position.x, home_y: mem.home_position.y,
                hunger: brain.hunger, fatigue: brain.fatigue, fear: brain.fear,
                social_drive: brain.social_drive, exploration_drive: brain.exploration_drive,
                maintenance_drive: brain.maintenance_drive, stress: brain.stress,
                agitation: brain.agitation,
                curiosity: traits.curiosity, aggression: traits.aggression,
                pheromone_sensitivity: traits.pheromone_sensitivity,
                chaos_tolerance: traits.chaos_tolerance, efficiency: traits.efficiency,
                speed_modifier: traits.speed_modifier,
            }
        }).collect();

        Self {
            version: 1,
            tick: sim.tick,
            width: sim.grid.width,
            height: sim.grid.height,
            cells,
            ants,
            rng_state: [0u8; 32],
        }
    }

    pub fn to_simulation(&self) -> Simulation {
        let mut sim = Simulation::new(self.width, self.height);
        sim.tick = self.tick;
        for (i, saved) in self.cells.iter().enumerate() {
            sim.grid.cells[i] = Cell {
                material: saved.material,
                humidity: saved.humidity,
                temperature: saved.temperature,
                stability: saved.stability,
                pheromones: saved.pheromones,
                organic_matter: saved.organic_matter,
            };
        }
        // Restore ants
        for sa in &self.ants {
            use crate::ant::{AntBody, AntBrain, AntMemory, AntTraits};
            let home = GridPos::new(sa.home_x, sa.home_y);
            sim.ants.bodies.push(AntBody::new(GridPos::new(sa.pos_x, sa.pos_y)));
            sim.ants.brains.push(AntBrain {
                hunger: sa.hunger, fatigue: sa.fatigue, fear: sa.fear,
                social_drive: sa.social_drive, exploration_drive: sa.exploration_drive,
                maintenance_drive: sa.maintenance_drive, stress: sa.stress,
                agitation: sa.agitation,
            });
            sim.ants.memories.push(AntMemory::new(home));
            sim.ants.traits_vec.push(AntTraits {
                curiosity: sa.curiosity, aggression: sa.aggression,
                pheromone_sensitivity: sa.pheromone_sensitivity, chaos_tolerance: sa.chaos_tolerance,
                efficiency: sa.efficiency, speed_modifier: sa.speed_modifier,
            });
        }
        sim
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::error::DecodeError> {
        bincode::serde::decode_from_slice(data, bincode::config::standard()).map(|(val, _)| val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Material;

    #[test]
    fn test_save_load_roundtrip() {
        let mut sim = Simulation::new(10, 10);
        sim.tick = 42;
        sim.grid.set_material(crate::grid::GridPos::new(3, 3), Material::Dirt);

        let save = SaveFile::from_simulation(&sim);
        let bytes = save.to_bytes().unwrap();
        let loaded = SaveFile::from_bytes(&bytes).unwrap();
        let restored = loaded.to_simulation();

        assert_eq!(restored.tick, 42);
        assert_eq!(restored.grid.width, 10);
        assert_eq!(restored.grid.height, 10);
        assert_eq!(restored.grid.get(crate::grid::GridPos::new(3, 3)).unwrap().material, Material::Dirt);
    }

    #[test]
    fn test_save_version_is_1() {
        let sim = Simulation::new(5, 5);
        let save = SaveFile::from_simulation(&sim);
        assert_eq!(save.version, 1);
    }

    #[test]
    fn test_invalid_bytes_returns_err() {
        let result = SaveFile::from_bytes(&[0xFF, 0xFF, 0xFF]);
        assert!(result.is_err());
    }
}
