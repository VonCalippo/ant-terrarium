use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ant Terrarium".into(),
                    resolution: (1024.0, 768.0).into(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(ant_renderer::TerrariumPlugin)
        .run();
}
