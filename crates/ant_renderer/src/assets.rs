use bevy::prelude::*;

pub const CELL_SIZE: f32 = 8.0;
pub const GRID_WIDTH: u16 = 128;
pub const GRID_HEIGHT: u16 = 96;
pub const QUEEN_COLOR: Color = Color::srgb(0.91, 0.75, 0.25);

pub fn material_color(material: ant_simulation::grid::Material) -> Color {
    match material {
        ant_simulation::grid::Material::Air => Color::srgb(0.227, 0.290, 0.416),
        ant_simulation::grid::Material::Dirt => Color::srgb(0.545, 0.412, 0.078),
        ant_simulation::grid::Material::LooseDirt => Color::srgb(0.678, 0.545, 0.235),
        ant_simulation::grid::Material::WetDirt => Color::srgb(0.38, 0.29, 0.16),
        ant_simulation::grid::Material::Sand => Color::srgb(0.761, 0.698, 0.502),
        ant_simulation::grid::Material::Stone => Color::srgb(0.408, 0.408, 0.408),
        ant_simulation::grid::Material::Water => Color::srgb(0.165, 0.353, 0.541),
        ant_simulation::grid::Material::Food => Color::srgb(0.91, 0.75, 0.25),
        ant_simulation::grid::Material::OrganicWaste => Color::srgb(0.4, 0.3, 0.2),
        ant_simulation::grid::Material::Fungus => Color::srgb(0.3, 0.6, 0.3),
        ant_simulation::grid::Material::Egg => Color::srgb(0.95, 0.95, 0.85),
        ant_simulation::grid::Material::Larva => Color::srgb(1.0, 0.9, 0.7),
    }
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
