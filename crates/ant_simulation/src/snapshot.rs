use crate::ant::AntEvent;
use crate::grid::{GridPos, Material, Direction};
use crate::terrain::TerrainEvent;
use crate::tick::Speed;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<CellSnapshot>,
    pub events: Vec<TerrainEvent>,
    pub ant_events: Vec<AntEvent>,
    pub ants: Vec<AntSnapshot>,
}

#[derive(Debug, Clone, Copy)]
pub struct CellSnapshot {
    pub material: Material,
    pub stability: u8,
    pub max_pheromone: (u8, u8), // (type_index 0-6, strength 0-255)
}

impl CellSnapshot {
    pub fn phero_strength(&self) -> u8 { self.max_pheromone.1 }
    pub fn phero_type(&self) -> u8 { self.max_pheromone.0 }
}

#[derive(Debug, Clone, Copy)]
pub struct AntSnapshot {
    pub id: usize,
    pub pos: GridPos,
    pub direction: Direction,
    pub carrying: Option<crate::ant::CarriedItem>,
    pub agitation: f32,
    pub action: crate::ant::Action,
    pub stress: f32,
}

#[derive(Debug, Clone)]
pub enum Command {
    AddFood { x: u16, y: u16 },
    AddWater { x: u16, y: u16 },
    ModifyTerrain { x: u16, y: u16, material: Material },
    SetSpeed(Speed),
}

impl Snapshot {
    pub fn from_simulation(sim: &crate::tick::Simulation) -> Self {
        let cells: Vec<CellSnapshot> = sim.grid.cells.iter().map(|c| {
            let p = c.pheromones;
            let types: [(u8, u8); 8] = [
                (0, p.food), (1, p.home), (2, p.danger), (3, p.dig),
                (4, p.queen), (5, p.death), (6, p.waste), (7, p.recruitment),
            ];
            let max = types.iter().max_by_key(|(_, v)| *v).copied().unwrap_or((0, 0));
            CellSnapshot {
                material: c.material,
                stability: c.stability,
                max_pheromone: max,
            }
        }).collect();

        let ants: Vec<AntSnapshot> = sim.ants.bodies.iter().enumerate().map(|(i, body)| {
            AntSnapshot {
                id: i,
                pos: body.pos,
                direction: body.direction,
                carrying: body.carrying,
                agitation: sim.ants.brains.get(i).map(|b| b.agitation).unwrap_or(0.0),
                action: body.current_action,
                stress: sim.ants.brains.get(i).map(|b| b.stress).unwrap_or(0.0),
            }
        }).collect();

        Self {
            tick: sim.tick,
            width: sim.grid.width,
            height: sim.grid.height,
            cells,
            events: sim.events.clone(),
            ant_events: sim.ant_events.clone(),
            ants,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick::Simulation;
    use crate::grid::{GridPos, Material};

    #[test]
    fn test_snapshot_from_simulation() {
        let mut sim = Simulation::new(5, 5);
        sim.tick();
        let snap = Snapshot::from_simulation(&sim);

        assert_eq!(snap.tick, 1);
        assert_eq!(snap.width, 5);
        assert_eq!(snap.height, 5);
        assert_eq!(snap.cells.len(), 25);
        assert!(snap.cells.iter().all(|c| c.material == Material::Air));
    }

    #[test]
    fn test_cell_snapshot_matches_grid() {
        let mut sim = Simulation::new(5, 5);
        sim.grid.set_material(GridPos::new(2, 2), Material::Dirt);
        let snap = Snapshot::from_simulation(&sim);

        let dirt_cell = snap.cells[2 * 5 + 2];
        assert_eq!(dirt_cell.material, Material::Dirt);
        assert_eq!(dirt_cell.stability, 128);
    }
}
