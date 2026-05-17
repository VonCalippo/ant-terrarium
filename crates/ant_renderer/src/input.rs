use bevy::prelude::*;
use ant_simulation::{
    grid::Material,
    snapshot::Command,
    tick::{Simulation, Speed},
};
use crate::assets::{CELL_SIZE, GRID_WIDTH, GRID_HEIGHT};

#[derive(Resource, Default)]
pub struct InputState {
    pub pending_commands: Vec<Command>,
}

fn save_path() -> std::path::PathBuf {
    let appdata = std::env::var("APPDATA")
        .unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(appdata)
        .join("ant_terrarium")
        .join("saves")
        .join("save_001.bin")
}

pub fn handle_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut simulation: ResMut<Simulation>,
    mut state: ResMut<InputState>,
) {
    state.pending_commands.clear();

    if keyboard.just_pressed(KeyCode::Space) {
        let new_speed = match simulation.speed {
            Speed::Paused => Speed::Normal,
            _ => Speed::Paused,
        };
        state.pending_commands.push(Command::SetSpeed(new_speed));
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        state.pending_commands.push(Command::SetSpeed(Speed::Normal));
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        state.pending_commands.push(Command::SetSpeed(Speed::Fast));
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        state.pending_commands.push(Command::SetSpeed(Speed::Fastest));
    }

    let Ok(window) = window.get_single() else { return };
    let cursor = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let Ok((camera, cam_transform)) = camera.get_single() else { return };

    let world_pos = match camera.viewport_to_world_2d(cam_transform, cursor) {
        Some(pos) => pos,
        None => return,
    };

    let grid_x = (world_pos.x / CELL_SIZE).floor();
    let grid_y = (-world_pos.y / CELL_SIZE).floor();

    if grid_x < 0.0 || grid_y < 0.0 || grid_x >= GRID_WIDTH as f32 || grid_y >= GRID_HEIGHT as f32 {
        return;
    }

    let x = grid_x as u16;
    let y = grid_y as u16;

    if mouse.just_pressed(MouseButton::Left) && !keyboard.pressed(KeyCode::ShiftLeft) {
        let cell = simulation.grid.get(ant_simulation::grid::GridPos::new(x, y));
        if let Some(cell) = cell {
            if cell.material == Material::Air && y == simulation.grid.surface_y() {
                state.pending_commands.push(Command::AddFood { x, y });
            }
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        let cell = simulation.grid.get(ant_simulation::grid::GridPos::new(x, y));
        if let Some(cell) = cell {
            if matches!(cell.material, Material::Dirt | Material::Sand) {
                state.pending_commands.push(Command::AddWater { x, y });
            }
        }
    }

    if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft) {
        if y <= 3 {
            let cell = simulation.grid.get(ant_simulation::grid::GridPos::new(x, y));
            if let Some(cell) = cell {
                let new_material = if cell.material == Material::Air { Material::Dirt } else { Material::Air };
                state.pending_commands.push(Command::ModifyTerrain { x, y, material: new_material });
            }
        }
    }

    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyS) {
        let save = ant_simulation::persistence::SaveFile::from_simulation(&simulation);
        if let Ok(bytes) = save.to_bytes() {
            let path = save_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, &bytes);
            info!("Saved to {:?}", path);
        }
    }

    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyL) {
        let path = save_path();
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(save) = ant_simulation::persistence::SaveFile::from_bytes(&bytes) {
                *simulation = save.to_simulation();
                info!("Loaded from {:?}", path);
            }
        }
    }

    for cmd in &state.pending_commands {
        match cmd {
            Command::AddFood { x, y } => {
                simulation.grid.set_material(
                    ant_simulation::grid::GridPos::new(*x, *y),
                    Material::Food,
                );
            }
            Command::AddWater { x, y } => {
                simulation.grid.set_material(
                    ant_simulation::grid::GridPos::new(*x, *y),
                    Material::WetDirt,
                );
            }
            Command::ModifyTerrain { x, y, material } => {
                simulation.grid.set_material(
                    ant_simulation::grid::GridPos::new(*x, *y),
                    *material,
                );
            }
            Command::SetSpeed(speed) => {
                simulation.set_speed(*speed);
            }
        }
    }
}

pub fn auto_save(
    time: Res<Time>,
    simulation: Res<Simulation>,
    mut timer: Local<Timer>,
) {
    if timer.duration().as_secs_f32() == 0.0 {
        *timer = Timer::from_seconds(60.0, TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        let save = ant_simulation::persistence::SaveFile::from_simulation(&simulation);
        if let Ok(bytes) = save.to_bytes() {
            let path = save_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, &bytes);
            info!("Auto-saved tick {}", simulation.tick);
        }
    }
}
