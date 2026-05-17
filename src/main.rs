use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            bevy::window::WindowPlugin {
                primary_window: Some(Window {
                    title: "Ant Terrarium".into(),
                    resolution: (1024.0, 768.0).into(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            },
            bevy::sprite::SpritePlugin::default(),
            bevy::ui::UiPlugin::default(),
            bevy::input::InputPlugin::default(),
        ))
        .add_plugins(ant_renderer::TerrariumPlugin)
        .run();
}
