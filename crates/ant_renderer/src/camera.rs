use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use crate::assets::{GRID_WIDTH, GRID_HEIGHT, CELL_SIZE};

pub fn setup_camera(mut commands: Commands) {
    let grid_center_x = GRID_WIDTH as f32 * CELL_SIZE / 2.0;
    let grid_center_y = -(GRID_HEIGHT as f32 * CELL_SIZE / 2.0);

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.02, 0.06)),
            ..default()
        },
        Transform::from_xyz(grid_center_x, grid_center_y, 100.0),
        CameraState {
            min_scale: 0.5,
            max_scale: 4.0,
            scale: 1.0,
        },
    ));
}

#[derive(Component)]
pub struct CameraState {
    pub min_scale: f32,
    pub max_scale: f32,
    pub scale: f32,
}

pub fn zoom_camera(
    mut query: Query<(&mut OrthographicProjection, &mut CameraState)>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    for event in scroll_events.read() {
        for (mut proj, mut state) in query.iter_mut() {
            let zoom_factor = 1.0 - event.y * 0.1;
            state.scale = (state.scale * zoom_factor).clamp(state.min_scale, state.max_scale);
            proj.scale = 1.0 / state.scale;
        }
    }
}

pub fn pan_camera(
    mut query: Query<&mut Transform, With<Camera2d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    state: Query<&CameraState>,
) {
    if !mouse.pressed(MouseButton::Right) {
        return;
    }

    let scale = state.single().scale;
    for event in motion.read() {
        for mut transform in query.iter_mut() {
            transform.translation.x -= event.delta.x * scale;
            transform.translation.y += event.delta.y * scale;
        }
    }
}

pub fn snap_to_queen(
    keyboard: Res<ButtonInput<KeyCode>>,
    simulation: Res<ant_simulation::tick::Simulation>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        let queen = simulation.grid.queen_position();
        for mut transform in query.iter_mut() {
            transform.translation.x = queen.x as f32 * CELL_SIZE;
            transform.translation.y = -(queen.y as f32 * CELL_SIZE);
        }
    }
}

pub fn keyboard_pan(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    state: Query<&CameraState>,
) {
    let scale = state.single().scale;
    let speed = 4.0 * scale;
    for mut transform in query.iter_mut() {
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            transform.translation.y += speed;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= speed;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= speed;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            transform.translation.x += speed;
        }
    }
}
