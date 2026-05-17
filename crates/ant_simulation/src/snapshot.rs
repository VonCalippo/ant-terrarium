use crate::grid::Material;
use crate::terrain::TerrainEvent;
use crate::tick::Speed;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<CellSnapshot>,
    pub events: Vec<TerrainEvent>,
}

#[derive(Debug, Clone, Copy)]
pub struct CellSnapshot {
    pub material: Material,
    pub stability: u8,
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
        let cells: Vec<CellSnapshot> = sim.grid.cells.iter().map(|c| CellSnapshot {
            material: c.material,
            stability: c.stability,
        }).collect();

        Self {
            tick: sim.tick,
            width: sim.grid.width,
            height: sim.grid.height,
            cells,
            events: sim.events.clone(),
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
