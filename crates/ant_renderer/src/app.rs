use bevy::prelude::*;
use ant_simulation::tick::Simulation;
use crate::{
    ants, widget,
    sprites::{self, SimulationState},
    input,
    hud::{self, HudState},
    camera,
};

#[derive(Resource)]
pub struct SimResource(pub Simulation);

impl std::ops::Deref for SimResource {
    type Target = Simulation;
    fn deref(&self) -> &Simulation { &self.0 }
}

impl std::ops::DerefMut for SimResource {
    fn deref_mut(&mut self) -> &mut Simulation { &mut self.0 }
}

impl SimResource {
    fn load_or_create() -> Self {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(appdata)
            .join("ant_terrarium")
            .join("saves")
            .join("save_001.bin");

        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(save) = ant_simulation::persistence::SaveFile::from_bytes(&bytes) {
                info!("Loaded save from {:?}", path);
                let mut sim = SimResource(save.to_simulation());
                if sim.ants.bodies.is_empty() {
                    sim.spawn_initial_ants(5);
                }
                return sim;
            }
        }

        info!("No save found, creating fresh world");
        let mut sim = SimResource(Simulation::from_grid(
            ant_simulation::grid::Grid::generate_initial_world(
                crate::assets::GRID_WIDTH,
                crate::assets::GRID_HEIGHT,
            )
        ));
        sim.spawn_initial_ants(5);
        sim
    }
}

pub struct TerrariumPlugin;

impl Plugin for TerrariumPlugin {
    fn build(&self, app: &mut App) {
        let simulation = SimResource::load_or_create();

        app.insert_resource(simulation);
        app.insert_resource(SimulationState::default());
        app.insert_resource(HudState::default());
        app.insert_resource(input::InputState::default());

        app.add_systems(Startup, (
            camera::setup_camera,
            sprites::setup_grid_sprites,
            ants::spawn_ant_sprites,
            sprites::setup_glass_overlay,
            hud::setup_hud,
        ));

        app.add_systems(Update, (
            input::handle_input,
            input::auto_save,
            sprites::tick_simulation,
            sprites::apply_snapshot,
            ants::update_ant_sprites,
            camera::zoom_camera,
            camera::pan_camera,
            camera::keyboard_pan,
            camera::snap_to_queen,
            hud::update_hud,
            widget::window_drag,
            widget::toggle_always_on_top,
            widget::escape_close,
        ));
    }
}
