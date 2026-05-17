use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    Air,
    LooseDirt,
    Dirt,
    WetDirt,
    Sand,
    Stone,
    Water,
    Food,
    OrganicWaste,
    Fungus,
    Egg,
    Larva,
}

impl Material {
    pub fn is_terrain(self) -> bool {
        matches!(self, Material::Dirt | Material::LooseDirt | Material::WetDirt | Material::Sand)
    }

    pub fn is_solid(self) -> bool {
        !matches!(self, Material::Air | Material::Water)
    }

    pub fn is_walkable(self) -> bool {
        !matches!(self, Material::Stone)
    }

    pub fn is_diggable(self) -> bool {
        matches!(self, Material::Dirt | Material::WetDirt | Material::Sand | Material::LooseDirt)
    }

    pub fn dig_ticks(self) -> Option<u8> {
        match self {
            Material::Dirt => Some(1),
            Material::WetDirt => Some(2),
            Material::Sand => Some(0),
            Material::LooseDirt => Some(0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PheromoneLayer {
    pub food: u8,
    pub home: u8,
    pub danger: u8,
    pub dig: u8,
    pub queen: u8,
    pub death: u8,
    pub waste: u8,
}

impl Default for PheromoneLayer {
    fn default() -> Self {
        Self { food: 0, home: 0, danger: 0, dig: 0, queen: 0, death: 0, waste: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PheromoneType {
    Food, Home, Danger, Dig, Queen, Death, Waste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPos {
    pub x: u16,
    pub y: u16,
}

impl GridPos {
    pub fn new(x: u16, y: u16) -> Self { Self { x, y } }

    pub fn neighbor(self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::N => self.y.checked_sub(1).map(|y| GridPos { y, ..self }),
            Direction::S => Some(GridPos { y: self.y + 1, ..self }),
            Direction::E => Some(GridPos { x: self.x + 1, ..self }),
            Direction::W => self.x.checked_sub(1).map(|x| GridPos { x, ..self }),
            Direction::NE => self.neighbor(Direction::N).and_then(|p| p.neighbor(Direction::E)),
            Direction::NW => self.neighbor(Direction::N).and_then(|p| p.neighbor(Direction::W)),
            Direction::SE => self.neighbor(Direction::S).and_then(|p| p.neighbor(Direction::E)),
            Direction::SW => self.neighbor(Direction::S).and_then(|p| p.neighbor(Direction::W)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N, S, E, W, NE, NW, SE, SW,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::N, Direction::S, Direction::E, Direction::W,
        Direction::NE, Direction::NW, Direction::SE, Direction::SW,
    ];

    pub const CARDINAL: [Direction; 4] = [
        Direction::N, Direction::S, Direction::E, Direction::W,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub material: Material,
    pub humidity: u8,
    pub temperature: i16,
    pub stability: u8,
    pub pheromones: PheromoneLayer,
    pub organic_matter: u8,
}

impl Cell {
    pub fn new(material: Material) -> Self {
        let (stability, humidity) = match material {
            Material::Stone => (255, 0),
            Material::Dirt => (128, 30),
            Material::WetDirt => (128, 180),
            Material::Sand => (64, 5),
            Material::LooseDirt => (64, 20),
            Material::Air => (0, 0),
            Material::Water => (0, 255),
            Material::Food => (0, 60),
            _ => (0, 0),
        };
        Self {
            material,
            humidity,
            temperature: 220,
            stability,
            pheromones: PheromoneLayer::default(),
            organic_matter: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        let len = width as usize * height as usize;
        Self {
            width,
            height,
            cells: vec![Cell::new(Material::Air); len],
        }
    }

    pub fn index(&self, pos: GridPos) -> usize {
        pos.y as usize * self.width as usize + pos.x as usize
    }

    pub fn contains(&self, pos: GridPos) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    pub fn get(&self, pos: GridPos) -> Option<&Cell> {
        if self.contains(pos) {
            Some(&self.cells[self.index(pos)])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, pos: GridPos) -> Option<&mut Cell> {
        if self.contains(pos) {
            let idx = self.index(pos);
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }

    pub fn set_material(&mut self, pos: GridPos, material: Material) -> bool {
        if let Some(cell) = self.get_mut(pos) {
            cell.material = material;
            cell.stability = Cell::new(material).stability;
            if material == Material::Air {
                cell.humidity = 0;
            }
            true
        } else {
            false
        }
    }

    pub fn cell_below(&self, pos: GridPos) -> Option<&Cell> {
        self.get(pos.neighbor(Direction::S)?)
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = GridPos> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| GridPos::new(x, y))
        })
    }

    pub fn surface_y(&self) -> u16 {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(GridPos::new(x, y)).map(|c| c.material != Material::Air).unwrap_or(false) {
                    return y;
                }
            }
        }
        self.height
    }

    pub fn generate_initial_world(width: u16, height: u16) -> Self {
        let mut grid = Self::new(width, height);
        let surface_y = height / 4;

        for x in 0..width {
            grid.set_material(GridPos::new(x, surface_y), Material::Dirt);
        }

        let stone_y = height - (height / 8);
        for y in (surface_y + 1)..stone_y {
            for x in 0..width {
                let material = if y % 6 == 0 && x % 4 == 0 {
                    if y % 12 == 0 { Material::Sand } else { Material::Dirt }
                } else {
                    Material::Dirt
                };
                grid.set_material(GridPos::new(x, y), material);
            }
        }

        for y in stone_y..height {
            for x in 0..width {
                grid.set_material(GridPos::new(x, y), Material::Stone);
            }
        }

        grid
    }

    pub fn queen_position(&self) -> GridPos {
        let y = self.surface_y();
        GridPos::new(self.width / 2, y)
    }

    pub fn evaporate_pheromones(&mut self) {
        const RATE: u8 = 1;
        for cell in &mut self.cells {
            cell.pheromones.food = cell.pheromones.food.saturating_sub(RATE);
            cell.pheromones.home = cell.pheromones.home.saturating_sub(RATE);
            cell.pheromones.danger = cell.pheromones.danger.saturating_sub(RATE);
            cell.pheromones.dig = cell.pheromones.dig.saturating_sub(RATE);
            cell.pheromones.queen = cell.pheromones.queen.saturating_sub(RATE);
            cell.pheromones.death = cell.pheromones.death.saturating_sub(RATE);
            cell.pheromones.waste = cell.pheromones.waste.saturating_sub(RATE);
        }
    }

    pub fn deposit_pheromone(&mut self, pos: GridPos, ptype: PheromoneType, amount: u8) {
        if let Some(cell) = self.get_mut(pos) {
            let target = match ptype {
                PheromoneType::Food => &mut cell.pheromones.food,
                PheromoneType::Home => &mut cell.pheromones.home,
                PheromoneType::Danger => &mut cell.pheromones.danger,
                PheromoneType::Dig => &mut cell.pheromones.dig,
                PheromoneType::Queen => &mut cell.pheromones.queen,
                PheromoneType::Death => &mut cell.pheromones.death,
                PheromoneType::Waste => &mut cell.pheromones.waste,
            };
            *target = target.saturating_add(amount);
        }
    }

    pub fn pheromone_at(&self, pos: GridPos, ptype: PheromoneType) -> u8 {
        self.get(pos).map(|c| match ptype {
            PheromoneType::Food => c.pheromones.food,
            PheromoneType::Home => c.pheromones.home,
            PheromoneType::Danger => c.pheromones.danger,
            PheromoneType::Dig => c.pheromones.dig,
            PheromoneType::Queen => c.pheromones.queen,
            PheromoneType::Death => c.pheromones.death,
            PheromoneType::Waste => c.pheromones.waste,
        }).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_is_terrain() {
        assert!(Material::Dirt.is_terrain());
        assert!(Material::Sand.is_terrain());
        assert!(Material::LooseDirt.is_terrain());
        assert!(Material::WetDirt.is_terrain());
        assert!(!Material::Air.is_terrain());
        assert!(!Material::Stone.is_terrain());
        assert!(!Material::Water.is_terrain());
    }

    #[test]
    fn test_material_is_solid() {
        assert!(Material::Dirt.is_solid());
        assert!(Material::Stone.is_solid());
        assert!(!Material::Air.is_solid());
        assert!(!Material::Water.is_solid());
    }

    #[test]
    fn test_material_is_walkable() {
        assert!(Material::Air.is_walkable());
        assert!(Material::Dirt.is_walkable());
        assert!(Material::Sand.is_walkable());
        assert!(!Material::Stone.is_walkable());
    }

    #[test]
    fn test_material_diggable() {
        assert!(Material::Dirt.is_diggable());
        assert!(Material::Sand.is_diggable());
        assert!(Material::WetDirt.is_diggable());
        assert!(!Material::Stone.is_diggable());
        assert!(!Material::Air.is_diggable());
    }

    #[test]
    fn test_material_dig_ticks() {
        assert_eq!(Material::Dirt.dig_ticks(), Some(1));
        assert_eq!(Material::WetDirt.dig_ticks(), Some(2));
        assert_eq!(Material::Sand.dig_ticks(), Some(0));
        assert_eq!(Material::Stone.dig_ticks(), None);
    }

    #[test]
    fn test_gridpos_neighbor_cardinals() {
        let pos = GridPos::new(5, 5);
        assert_eq!(pos.neighbor(Direction::N), Some(GridPos::new(5, 4)));
        assert_eq!(pos.neighbor(Direction::S), Some(GridPos::new(5, 6)));
        assert_eq!(pos.neighbor(Direction::E), Some(GridPos::new(6, 5)));
        assert_eq!(pos.neighbor(Direction::W), Some(GridPos::new(4, 5)));
    }

    #[test]
    fn test_gridpos_neighbor_boundary() {
        let pos = GridPos::new(0, 0);
        assert_eq!(pos.neighbor(Direction::N), None);
        assert_eq!(pos.neighbor(Direction::W), None);
        assert_eq!(pos.neighbor(Direction::NW), None);
    }

    #[test]
    fn test_pheromone_layer_default() {
        let p = PheromoneLayer::default();
        assert_eq!(p.food, 0);
        assert_eq!(p.home, 0);
        assert_eq!(p.waste, 0);
    }

    #[test]
    fn test_cell_new_defaults() {
        let dirt = Cell::new(Material::Dirt);
        assert_eq!(dirt.material, Material::Dirt);
        assert_eq!(dirt.stability, 128);
        assert_eq!(dirt.humidity, 30);

        let air = Cell::new(Material::Air);
        assert_eq!(air.stability, 0);
        assert_eq!(air.humidity, 0);
    }

    #[test]
    fn test_grid_new_correct_size() {
        let grid = Grid::new(128, 96);
        assert_eq!(grid.width, 128);
        assert_eq!(grid.height, 96);
        assert_eq!(grid.cells.len(), 128 * 96);
        assert!(grid.cells.iter().all(|c| c.material == Material::Air));
    }

    #[test]
    fn test_grid_index() {
        let grid = Grid::new(3, 3);
        assert_eq!(grid.index(GridPos::new(0, 0)), 0);
        assert_eq!(grid.index(GridPos::new(2, 0)), 2);
        assert_eq!(grid.index(GridPos::new(0, 1)), 3);
        assert_eq!(grid.index(GridPos::new(2, 2)), 8);
    }

    #[test]
    fn test_grid_contains() {
        let grid = Grid::new(10, 10);
        assert!(grid.contains(GridPos::new(0, 0)));
        assert!(grid.contains(GridPos::new(9, 9)));
        assert!(!grid.contains(GridPos::new(10, 0)));
        assert!(!grid.contains(GridPos::new(0, 10)));
    }

    #[test]
    fn test_grid_get_and_set() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(3, 3), Material::Dirt);
        let cell = grid.get(GridPos::new(3, 3)).unwrap();
        assert_eq!(cell.material, Material::Dirt);
        assert_eq!(cell.stability, 128);
        assert!(grid.get(GridPos::new(100, 100)).is_none());
    }

    #[test]
    fn test_grid_cell_below() {
        let mut grid = Grid::new(10, 10);
        grid.set_material(GridPos::new(5, 4), Material::Dirt);
        let below = grid.cell_below(GridPos::new(5, 3));
        assert!(below.is_some());
        assert_eq!(below.unwrap().material, Material::Dirt);
    }

    #[test]
    fn test_grid_surface_y_all_air() {
        let grid = Grid::new(10, 10);
        assert_eq!(grid.surface_y(), 10); // no terrain, returns height
    }

    #[test]
    fn test_grid_iter_positions() {
        let grid = Grid::new(4, 3);
        let positions: Vec<_> = grid.iter_positions().collect();
        assert_eq!(positions.len(), 12);
        assert_eq!(positions[0], GridPos::new(0, 0));
        assert_eq!(positions[11], GridPos::new(3, 2));
    }

    #[test]
    fn test_pheromone_evaporation() {
        let mut grid = Grid::new(10, 10);
        grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 200);
        assert_eq!(grid.pheromone_at(GridPos::new(5, 5), PheromoneType::Food), 200);
        grid.evaporate_pheromones();
        assert_eq!(grid.pheromone_at(GridPos::new(5, 5), PheromoneType::Food), 199);
    }

    #[test]
    fn test_pheromone_deposit_saturates() {
        let mut grid = Grid::new(10, 10);
        grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 250);
        grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 20);
        assert_eq!(grid.pheromone_at(GridPos::new(5, 5), PheromoneType::Food), 255);
    }

    #[test]
    fn test_pheromone_evaporates_to_zero() {
        let mut grid = Grid::new(10, 10);
        grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 1);
        grid.evaporate_pheromones();
        assert_eq!(grid.pheromone_at(GridPos::new(5, 5), PheromoneType::Food), 0);
    }

    #[test]
    fn test_pheromone_multiple_types() {
        let mut grid = Grid::new(10, 10);
        grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Food, 100);
        grid.deposit_pheromone(GridPos::new(5, 5), PheromoneType::Home, 50);
        assert_eq!(grid.pheromone_at(GridPos::new(5, 5), PheromoneType::Food), 100);
        assert_eq!(grid.pheromone_at(GridPos::new(5, 5), PheromoneType::Home), 50);
    }

    #[test]
    fn test_generate_initial_world() {
        let grid = Grid::generate_initial_world(128, 96);
        assert_eq!(grid.width, 128);
        assert_eq!(grid.height, 96);
        assert_eq!(grid.get(GridPos::new(64, 0)).unwrap().material, Material::Air);
        let sy = grid.surface_y();
        assert_eq!(grid.get(GridPos::new(64, sy)).unwrap().material, Material::Dirt);
        assert_eq!(grid.get(GridPos::new(64, 95)).unwrap().material, Material::Stone);
        let qpos = grid.queen_position();
        assert_eq!(qpos.x, 64);
        assert_eq!(qpos.y, sy);
    }
}
