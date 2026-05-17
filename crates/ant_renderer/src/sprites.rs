use bevy::prelude::*;
use ant_simulation::{
    snapshot::Snapshot,
    tick::Simulation,
};
use crate::assets::{self, CELL_SIZE, GRID_WIDTH, GRID_HEIGHT, QUEEN_COLOR};

#[derive(Resource)]
pub struct SimulationState {
    pub snapshot: Option<Snapshot>,
    pub tick_timer: Timer,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            snapshot: None,
            tick_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
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

pub fn setup_grid_sprites(
    mut commands: Commands,
    simulation: Res<Simulation>,
) {
    let snap = Snapshot::from_simulation(&simulation);

    for y in 0..snap.height {
        for x in 0..snap.width {
            let idx = y as usize * snap.width as usize + x as usize;
            let cell = snap.cells[idx];
            let color = assets::material_color(cell.material);
            let world_x = x as f32 * CELL_SIZE;
            let world_y = -(y as f32 * CELL_SIZE);

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 0.0),
                CellSprite { grid_x: x as u16, grid_y: y as u16 },
            ));
        }
    }

    let queen = simulation.grid.queen_position();
    commands.spawn((
        Sprite {
            color: QUEEN_COLOR,
            custom_size: Some(Vec2::splat(CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            queen.x as f32 * CELL_SIZE,
            -(queen.y as f32 * CELL_SIZE),
            1.0,
        ),
        QueenMarker,
    ));
}

pub fn setup_glass_overlay(mut commands: Commands) {
    let width_px = GRID_WIDTH as f32 * CELL_SIZE;
    let height_px = GRID_HEIGHT as f32 * CELL_SIZE;
    let center_x = width_px / 2.0;
    let center_y = -height_px / 2.0;

    let border_color = Color::srgba(0.23, 0.23, 0.35, 1.0);
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
    mut simulation: ResMut<Simulation>,
    mut state: ResMut<SimulationState>,
) {
    let interval_ms = match simulation.tick_interval_ms() {
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

            if cell.stability < 64 {
                let alpha = cell.stability as f32 / 64.0;
                color.set_alpha(alpha.max(0.2));
            }

            sprite.color = color;
        }
    }
}
