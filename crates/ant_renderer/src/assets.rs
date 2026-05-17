use bevy::prelude::*;

pub const CELL_SIZE: f32 = 8.0;
pub const GRID_WIDTH: u16 = 128;
pub const GRID_HEIGHT: u16 = 96;
pub const QUEEN_COLOR: Color = Color::srgb(0.91, 0.75, 0.25);

pub fn material_color(material: ant_simulation::grid::Material) -> Color {
    match material {
        ant_simulation::grid::Material::Air => Color::srgb(0.50, 0.65, 0.85),    // sky blue
        ant_simulation::grid::Material::Dirt => Color::srgb(0.35, 0.22, 0.08),    // dark brown
        ant_simulation::grid::Material::LooseDirt => Color::srgb(0.55, 0.38, 0.18), // lighter brown
        ant_simulation::grid::Material::WetDirt => Color::srgb(0.22, 0.16, 0.08),  // very dark wet
        ant_simulation::grid::Material::Sand => Color::srgb(0.75, 0.65, 0.40),     // sandy tan
        ant_simulation::grid::Material::Stone => Color::srgb(0.30, 0.30, 0.33),    // dark gray
        ant_simulation::grid::Material::Water => Color::srgb(0.10, 0.30, 0.60),    // deep blue
        ant_simulation::grid::Material::Food => Color::srgb(0.95, 0.80, 0.20),     // bright amber
        ant_simulation::grid::Material::OrganicWaste => Color::srgb(0.28, 0.18, 0.10), // dark organic
        ant_simulation::grid::Material::Fungus => Color::srgb(0.20, 0.50, 0.25),   // green fungus
        ant_simulation::grid::Material::Egg => Color::srgb(0.95, 0.95, 0.80),      // cream
        ant_simulation::grid::Material::Larva => Color::srgb(1.0, 0.88, 0.65),     // warm cream
    }
}

pub fn surface_color() -> Color {
    Color::srgb(0.25, 0.50, 0.15) // rich grass green
}

pub fn sky_color(y: u16, total: u16) -> Color {
    let t = y as f32 / total as f32;
    Color::srgb(0.45 + t * 0.15, 0.60 + t * 0.15, 0.78 + t * 0.15)
}

pub fn ant_body_color(agitation: f32, carrying: Option<ant_simulation::ant::CarriedItem>) -> Color {
    let base = Color::srgb(0.2, 0.15, 0.1);
    let mut agitated = Color::srgb(0.35, 0.25, 0.15);
    if let Some(item) = carrying {
        agitated = agitated.mix(&ant_carrying_tint(item), 0.5);
    }
    base.mix(&agitated, (agitation + 0.2).min(1.0))
}

pub fn ant_carrying_tint(item: ant_simulation::ant::CarriedItem) -> Color {
    match item {
        ant_simulation::ant::CarriedItem::Dirt => Color::srgb(0.678, 0.545, 0.235),
        ant_simulation::ant::CarriedItem::Food => Color::srgb(0.91, 0.75, 0.25),
    }
}
