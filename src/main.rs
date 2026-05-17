use bevy::{prelude::*, window::WindowLevel};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ant Terrarium".into(),
                    resolution: (800.0, 600.0).into(),
                    resizable: true,
                    decorations: false,
                    window_level: WindowLevel::AlwaysOnTop,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(ant_renderer::TerrariumPlugin)
        .run();
}
