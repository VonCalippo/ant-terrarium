use crate::grid::{Grid, GridPos, Material};

#[derive(Debug, Clone)]
pub struct DigState {
    pub target: GridPos,
    pub ticks_remaining: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainEvent {
    DigProgress { pos: GridPos, remaining: u8 },
    DigComplete { pos: GridPos },
    Collapse { pos: GridPos, from: Material, to: Material },
    CollapseChain { positions: Vec<GridPos> },
}

const STABILITY_DECAY: u8 = 4;
const STABILITY_RECOVERY: u8 = 2;

pub fn process_digging(grid: &mut Grid, pending: &mut Vec<DigState>) -> Vec<TerrainEvent> {
    let mut events = Vec::new();
    let mut completed = Vec::new();

    for (i, dig) in pending.iter_mut().enumerate() {
        if dig.ticks_remaining == 0 {
            grid.set_material(dig.target, Material::Air);
            events.push(TerrainEvent::DigComplete { pos: dig.target });
            completed.push(i);
        } else {
            dig.ticks_remaining -= 1;
            if dig.ticks_remaining == 0 {
                let cell = grid.get(dig.target);
                if let Some(cell) = cell {
                    let next = match cell.material {
                        Material::WetDirt => Material::LooseDirt,
                        _ => Material::Air,
                    };
                    grid.set_material(dig.target, next);
                    if next == Material::Air {
                        events.push(TerrainEvent::DigComplete { pos: dig.target });
                    } else {
                        events.push(TerrainEvent::DigProgress { pos: dig.target, remaining: 0 });
                    }
                }
                completed.push(i);
            } else {
                events.push(TerrainEvent::DigProgress { pos: dig.target, remaining: dig.ticks_remaining });
            }
        }
    }

    for i in completed.into_iter().rev() {
        pending.remove(i);
    }

    events
}

pub fn start_dig(grid: &Grid, pos: GridPos, pending: &mut Vec<DigState>) -> bool {
    let cell = match grid.get(pos) {
        Some(c) => c,
        None => return false,
    };

    if let Some(ticks) = cell.material.dig_ticks() {
        if pending.iter().any(|d| d.target == pos) {
            return false;
        }
        pending.push(DigState { target: pos, ticks_remaining: ticks });
        true
    } else {
        false
    }
}

pub fn update_stability(grid: &mut Grid) -> Vec<TerrainEvent> {
    let mut events = Vec::new();
    let width = grid.width;
    let height = grid.height;

    for y in (0..height).rev() {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            let material = match grid.get(pos) {
                Some(c) => c.material,
                None => continue,
            };

            if !material.is_terrain() {
                continue;
            }

            // Phase 1: read all needed data immutably
            let has_support = match grid.cell_below(pos) {
                Some(below) => below.material.is_solid(),
                None => false,
            };
            let pos_below = GridPos::new(pos.x, pos.y + 1);
            let below_is_unsupportive = grid.get(pos_below)
                .map(|b| !b.material.is_solid())
                .unwrap_or(false);

            // Phase 2: apply mutations
            let cell = grid.get_mut(pos).unwrap();

            if has_support || y == height - 1 {
                cell.stability = cell.stability.saturating_add(STABILITY_RECOVERY).min(255);
            } else if cell.stability > 0 {
                let before = cell.stability;
                cell.stability = cell.stability.saturating_sub(STABILITY_DECAY);

                if cell.stability == 0 && before > 0 {
                    let from = cell.material;
                    cell.material = Material::LooseDirt;
                    events.push(TerrainEvent::Collapse { pos, from, to: Material::LooseDirt });
                }
            }

            if cell.material == Material::LooseDirt && cell.stability == 0 && below_is_unsupportive {
                cell.material = Material::Air;
                events.push(TerrainEvent::Collapse { pos, from: Material::LooseDirt, to: Material::Air });
            }
        }
    }

    if !events.is_empty() {
        let positions: Vec<GridPos> = events.iter().filter_map(|e| match e {
            TerrainEvent::Collapse { pos, .. } => Some(*pos),
            _ => None,
        }).collect();
        if positions.len() > 1 {
            events.push(TerrainEvent::CollapseChain { positions });
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Grid, Material, GridPos};

    #[test]
    fn test_start_dig_dirt() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Dirt);
        let mut pending = Vec::new();

        assert!(start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].target, GridPos::new(5, 5));
        assert_eq!(pending[0].ticks_remaining, 1);
    }

    #[test]
    fn test_start_dig_stone_fails() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Stone);
        let mut pending = Vec::new();

        assert!(!start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert!(pending.is_empty());
    }

    #[test]
    fn test_start_dig_no_duplicate() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Dirt);
        let mut pending = Vec::new();

        assert!(start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert!(!start_dig(&grid, GridPos::new(5, 5), &mut pending));
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_process_digging_single_step() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Dirt);
        let mut pending = vec![DigState { target: GridPos::new(5, 5), ticks_remaining: 1 }];

        let events = process_digging(&mut grid, &mut pending);

        assert!(pending.is_empty());
        assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().material, Material::Air);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TerrainEvent::DigComplete { .. }));
    }

    #[test]
    fn test_process_digging_wet_dirt_two_step() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::WetDirt);
        let mut pending = vec![DigState { target: GridPos::new(5, 5), ticks_remaining: 2 }];

        let _events = process_digging(&mut grid, &mut pending);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].ticks_remaining, 1);

        let _events2 = process_digging(&mut grid, &mut pending);
        assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().material, Material::LooseDirt);

        start_dig(&grid, GridPos::new(5, 5), &mut pending);
        let _events3 = process_digging(&mut grid, &mut pending);
        assert_eq!(grid.get(GridPos::new(5, 5)).unwrap().material, Material::Air);
    }

    #[test]
    fn test_unsupported_dirt_collapses() {
        let mut grid = Grid::new(5, 10);
        grid.set_material(GridPos::new(2, 5), Material::Dirt);
        assert_eq!(grid.get(GridPos::new(2, 6)).unwrap().material, Material::Air);

        for _ in 0..40 {
            update_stability(&mut grid);
        }

        assert_eq!(grid.get(GridPos::new(2, 5)).unwrap().material, Material::Air);
    }

    #[test]
    fn test_supported_dirt_stays() {
        let mut grid = Grid::new(5, 10);
        grid.set_material(GridPos::new(2, 6), Material::Stone);
        grid.set_material(GridPos::new(2, 5), Material::Dirt);

        for _ in 0..100 {
            update_stability(&mut grid);
        }

        assert_eq!(grid.get(GridPos::new(2, 5)).unwrap().material, Material::Dirt);
        assert_eq!(grid.get(GridPos::new(2, 5)).unwrap().stability, 255);
    }
}
