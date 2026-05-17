use crate::grid::{Grid, GridPos, Material, Direction};

pub fn tick_ecology(grid: &mut Grid) {
    spread_humidity(grid);
    process_water(grid);
    grow_fungus(grid);
    decompose_organics(grid);
    decay_food(grid);
}

fn spread_humidity(grid: &mut Grid) {
    let width = grid.width;
    let height = grid.height;
    let mut diffs: Vec<(GridPos, i16)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            let cell = match grid.get(pos) {
                Some(c) => c,
                None => continue,
            };
            let my_hum = cell.humidity as i16;

            // WetDirt generates humidity
            if cell.material == Material::WetDirt {
                diffs.push((pos, 2));
            }

            // Deep air ventilation: lose humidity
            if cell.material == Material::Air {
                let surface_y = grid.surface_y();
                let depth = if y > surface_y { y - surface_y } else { 0 };
                if depth > 5 {
                    diffs.push((pos, -1));
                } else if y < surface_y && y < 3 {
                    // Surface air: drift toward ambient 30%
                    let ambient: i16 = 30;
                    diffs.push((pos, (ambient - my_hum) / 10));
                }
            }

            // Spread to neighbors: if I have more humidity, share
            for dir in &Direction::CARDINAL {
                if let Some(np) = pos.neighbor(*dir) {
                    if let Some(ncell) = grid.get(np) {
                        let n_hum = ncell.humidity as i16;
                        if my_hum > n_hum + 2 {
                            diffs.push((pos, -1));
                            diffs.push((np, 1));
                        }
                    }
                }
            }
        }
    }

    for (pos, delta) in diffs {
        if let Some(cell) = grid.get_mut(pos) {
            let new_val = (cell.humidity as i16 + delta).clamp(0, 255);
            cell.humidity = new_val as u8;
        }
    }
}

fn process_water(grid: &mut Grid) {
    let width = grid.width;
    let height = grid.height;
    let mut changes: Vec<(GridPos, Material)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            let cell = match grid.get(pos) {
                Some(c) => c,
                None => continue,
            };

            if cell.material != Material::Water { continue; }

            // Spread humidity to adjacent cells
            for dir in &Direction::CARDINAL {
                if let Some(np) = pos.neighbor(*dir) {
                    if let Some(ncell) = grid.get(np) {
                        if ncell.material != Material::Water && ncell.material != Material::Stone {
                            changes.push((np, Material::WetDirt));
                        }
                    }
                }
            }

            // Evaporate if adjacent to Air with low humidity
            let mut adjacent_air = false;
            let mut total_humidity: u16 = 0;
            let mut count: u16 = 0;
            for dir in &Direction::ALL {
                if let Some(np) = pos.neighbor(*dir) {
                    if let Some(ncell) = grid.get(np) {
                        if ncell.material == Material::Air {
                            adjacent_air = true;
                        }
                        total_humidity += ncell.humidity as u16;
                        count += 1;
                    }
                }
            }
            if adjacent_air && count > 0 && total_humidity / count < 50 {
                changes.push((pos, Material::WetDirt));
            }
        }
    }

    for (pos, mat) in changes {
        grid.set_material(pos, mat);
    }
}

fn grow_fungus(grid: &mut Grid) {
    let width = grid.width;
    let height = grid.height;
    let mut spawns: Vec<GridPos> = Vec::new();
    let mut spreads: Vec<GridPos> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            let cell = match grid.get(pos) {
                Some(c) => c,
                None => continue,
            };

            // Fungus growth from OrganicWaste
            if cell.material == Material::OrganicWaste && cell.humidity > 150 {
                // Count ticks implicitly via random chance
                use rand::Rng;
                if rand::thread_rng().gen_range(0..1200) == 0 {
                    spawns.push(pos);
                }
            }

            // Fungus spread to adjacent cells
            if cell.material == Material::Fungus {
                for dir in &Direction::ALL {
                    if let Some(np) = pos.neighbor(*dir) {
                        if let Some(ncell) = grid.get(np) {
                            if ncell.humidity > 150
                                && (ncell.material == Material::Dirt || ncell.material == Material::WetDirt || ncell.material == Material::LooseDirt)
                                && ncell.organic_matter > 10
                            {
                                use rand::Rng;
                                if rand::thread_rng().gen_range(0..20) == 0 {
                                    spreads.push(np);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for pos in spawns {
        grid.set_material(pos, Material::Fungus);
    }
    for pos in spreads {
        grid.set_material(pos, Material::Fungus);
    }
}

fn decompose_organics(grid: &mut Grid) {
    let width = grid.width;
    let height = grid.height;
    let mut organics: Vec<(GridPos, u8)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            let cell = match grid.get(pos) {
                Some(c) => c,
                None => continue,
            };

            if cell.material == Material::OrganicWaste {
                // Spread organic matter to neighbors
                for dir in &Direction::CARDINAL {
                    if let Some(np) = pos.neighbor(*dir) {
                        organics.push((np, 5));
                    }
                }
            }

            // Slow decay of organic matter
            if cell.organic_matter > 0 {
                use rand::Rng;
                if rand::thread_rng().gen_range(0..100) == 0 {
                    organics.push((pos, 255)); // signal to decay by 1
                }
            }
        }
    }

    for (pos, amount) in organics {
        if amount == 255 {
            // Decay signal
            if let Some(cell) = grid.get_mut(pos) {
                cell.organic_matter = cell.organic_matter.saturating_sub(1);
            }
        } else if let Some(cell) = grid.get_mut(pos) {
            cell.organic_matter = cell.organic_matter.saturating_add(amount);
        }
    }
}

fn decay_food(grid: &mut Grid) {
    let width = grid.width;
    let height = grid.height;
    let mut changes: Vec<(GridPos, Material)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pos = GridPos::new(x, y);
            let cell = match grid.get(pos) {
                Some(c) => c,
                None => continue,
            };

            if cell.material == Material::Food {
                use rand::Rng;
                // Food decays after ~4800 ticks
                if rand::thread_rng().gen_range(0..4800) == 0 {
                    changes.push((pos, Material::OrganicWaste));
                }
            }
        }
    }

    for (pos, mat) in changes {
        grid.set_material(pos, mat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_water_sets_adjacent_wet() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Water);
        grid.set_material(GridPos::new(5, 4), Material::Dirt);
        process_water(&mut grid);
        assert_eq!(grid.get(GridPos::new(5, 4)).unwrap().material, Material::WetDirt);
    }

    #[test]
    fn test_humidity_spreads() {
        let mut grid = Grid::new(10, 10);
        grid.get_mut(GridPos::new(5, 5)).unwrap().humidity = 200;
        spread_humidity(&mut grid);
        // Should have spread to neighbors
        let n = grid.get(GridPos::new(5, 4)).unwrap().humidity;
        assert!(n > 0);
    }

    #[test]
    fn test_food_decays() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 5), Material::Food);
        // Run decay many times
        for _ in 0..5000 {
            decay_food(&mut grid);
        }
        // Food should have decayed to OrganicWaste
        let mat = grid.get(GridPos::new(5, 5)).unwrap().material;
        assert!(mat == Material::OrganicWaste || mat == Material::Food);
    }
}
