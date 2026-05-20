use bevy::prelude::*;
use ant_simulation::snapshot::Snapshot;
use crate::app::SimResource;
use crate::pixelart::PixelAssets;
use crate::textures::{TextureAssets, texture_for_material, grass_variant, tunnel_texture};
use crate::assets::{self, CELL_SIZE, GRID_WIDTH, GRID_HEIGHT};

// Sky colors now in assets::sky_color()

#[derive(Resource)]
pub struct SimulationState {
    pub snapshot: Option<Snapshot>,
    pub tick_timer: Timer,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            snapshot: None,
            tick_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct CellSprite {
    pub grid_x: u16,
    pub grid_y: u16,
}

#[derive(Component)]
pub struct QueenMarker;

#[derive(Component)]
pub struct GlassOverlay;

#[derive(Component)]
pub struct SkyBackground;

pub fn setup_sky_background(
    _commands: Commands,
    _pixel_assets: Res<PixelAssets>,
) {
    // Sky is rendered per-cell via the material color in setup_grid_sprites
    // Air cells get a sky gradient color based on their y position
}

pub fn setup_grid_sprites(
    mut commands: Commands,
    simulation: Res<SimResource>,
    pixel_assets: Res<PixelAssets>,
    textures: Res<TextureAssets>,
) {
    let snap = Snapshot::from_simulation(&simulation);
    let surface_y = simulation.grid.surface_y();

    for y in 0..snap.height {
        for x in 0..snap.width {
            let idx = y as usize * snap.width as usize + x as usize;
            let cell = snap.cells[idx];
            let tex = match cell.material {
                ant_simulation::grid::Material::Dirt if y == surface_y => Some(grass_variant(&textures, x as u16, y as u16)),
                ant_simulation::grid::Material::Dirt
                | ant_simulation::grid::Material::Stone
                | ant_simulation::grid::Material::Sand
                | ant_simulation::grid::Material::LooseDirt
                | ant_simulation::grid::Material::WetDirt => texture_for_material(&textures, cell.material, x as u16, y as u16),
                ant_simulation::grid::Material::Air if y > surface_y => Some(tunnel_texture(&textures)),
                ant_simulation::grid::Material::Egg => Some(pixel_assets.egg_sprite.clone()),
                ant_simulation::grid::Material::Larva => Some(pixel_assets.larva_sprite.clone()),
                ant_simulation::grid::Material::Fungus => Some(pixel_assets.fungus_sprite.clone()),
                _ => None,
            };

            let world_x = x as f32 * CELL_SIZE;
            let world_y = -(y as f32 * CELL_SIZE);

            if let Some(image) = tex {
                commands.spawn((
                    Sprite { image, custom_size: Some(Vec2::splat(CELL_SIZE)), ..default() },
                    Transform::from_xyz(world_x, world_y, 0.0),
                    CellSprite { grid_x: x as u16, grid_y: y as u16 },
                ));
            } else {
                let color = match cell.material {
                    ant_simulation::grid::Material::Air => assets::sky_color(y as u16, snap.height),
                    _ => assets::material_color(cell.material),
                };
                commands.spawn((
                    Sprite { color, custom_size: Some(Vec2::splat(CELL_SIZE)), ..default() },
                    Transform::from_xyz(world_x, world_y, 0.0),
                    CellSprite { grid_x: x as u16, grid_y: y as u16 },
                ));
            }
        }
    }

    // Queen with pixel art sprite
    let queen_pos = simulation.grid.queen_position();
    commands.spawn((
        Sprite {
            image: pixel_assets.queen_sprite.clone(),
            custom_size: Some(Vec2::splat(CELL_SIZE * 1.5)),
            ..default()
        },
        Transform::from_xyz(
            queen_pos.x as f32 * CELL_SIZE,
            -(queen_pos.y as f32 * CELL_SIZE),
            2.0,
        ),
        QueenMarker,
    ));
}

pub fn setup_glass_overlay(mut commands: Commands) {
    let width_px = GRID_WIDTH as f32 * CELL_SIZE;
    let height_px = GRID_HEIGHT as f32 * CELL_SIZE;
    let center_x = width_px / 2.0;
    let center_y = -height_px / 2.0;

    let border_color = Color::srgba(0.35, 0.35, 0.45, 1.0); // lighter glass frame
    let border_thickness = 3.0;

    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(width_px, border_thickness)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y - height_px / 2.0, 5.0),
    ));

    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(width_px, border_thickness)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y + height_px / 2.0, 5.0),
    ));

    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(border_thickness, height_px)),
            ..default()
        },
        Transform::from_xyz(center_x - width_px / 2.0, center_y, 5.0),
    ));

    commands.spawn((
        GlassOverlay,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(border_thickness, height_px)),
            ..default()
        },
        Transform::from_xyz(center_x + width_px / 2.0, center_y, 5.0),
    ));

    commands.spawn((
        GlassOverlay,
        Sprite {
            color: Color::srgba(0.59, 0.78, 1.0, 0.08),
            custom_size: Some(Vec2::new(width_px, height_px * 0.30)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y - height_px * 0.35, 5.0),
    ));
}

pub fn tick_simulation(
    time: Res<Time>,
    mut simulation: ResMut<SimResource>,
    mut state: ResMut<SimulationState>,
) {
    let _interval_ms = match simulation.tick_interval_ms() {
        Some(ms) => ms as f32 / 1000.0,
        None => return,
    };

    state.tick_timer.tick(time.delta());
    if state.tick_timer.just_finished() {
        simulation.tick();
        state.snapshot = Some(Snapshot::from_simulation(&simulation));
    }
}

pub fn apply_snapshot(
    mut query: Query<(&CellSprite, &mut Sprite)>,
    simulation_state: Res<SimulationState>,
) {
    let snapshot = match &simulation_state.snapshot {
        Some(s) => s,
        None => return,
    };

    for (cell_sprite, mut sprite) in query.iter_mut() {
        let idx = cell_sprite.grid_y as usize * snapshot.width as usize + cell_sprite.grid_x as usize;
        if let Some(cell) = snapshot.cells.get(idx) {
            let mut color = assets::material_color(cell.material);

            // Pheromone overlay
            let ps = cell.phero_strength();
            if ps > 15 {
                let pc = phero_color(cell.phero_type(), ps);
                color = color.mix(&pc, (ps as f32 / 255.0) * 0.5);
            }

            if cell.stability < 64 {
                let alpha = cell.stability as f32 / 64.0;
                color.set_alpha(alpha.max(0.3));
            }

            if cell.material == ant_simulation::grid::Material::Air {
                color.set_alpha(0.65);
            }

            sprite.color = color;
        }
    }
}

fn phero_color(ptype: u8, s: u8) -> Color {
    let v = s as f32 / 255.0;
    match ptype {
        0 => Color::srgb(0.1, 0.6 + v * 0.4, 0.1),     // Food = green
        1 => Color::srgb(0.1, 0.2 + v * 0.5, 0.7 + v * 0.3), // Home = blue
        2 => Color::srgb(0.9 + v * 0.1, 0.1, 0.1),       // Danger = red
        3 => Color::srgb(0.7, 0.6 + v * 0.3, 0.1),       // Dig = yellow
        4 => Color::srgb(0.9, 0.75 + v * 0.2, 0.15),     // Queen = gold
        5 => Color::srgb(0.5, 0.05, 0.25 + v * 0.3),     // Death = dark purple
        6 => Color::srgb(0.4 + v * 0.2, 0.3, 0.18),       // Waste = brown
        7 => Color::srgb(1.0, 0.5 + v * 0.3, 0.1),         // Recruitment = orange
        _ => Color::WHITE,
    }
}
