use bevy::prelude::*;
use crate::app::SimResource;
use crate::sprites::SimulationState;
use crate::pixelart::{PixelAssets, ant_sprite_handle};
use crate::assets::CELL_SIZE;

#[derive(Component)]
pub struct AntSprite {
    pub ant_id: usize,
}

pub fn spawn_ant_sprites(
    mut commands: Commands,
    simulation: Res<SimResource>,
    pixel_assets: Res<PixelAssets>,
) {
    let snap = ant_simulation::snapshot::Snapshot::from_simulation(&simulation);

    for ant in &snap.ants {
        let image = ant_sprite_handle(&pixel_assets, ant.direction);
        let tint = agitation_tint(ant.agitation);

        commands.spawn((
            Sprite {
                image,
                color: tint,
                custom_size: Some(Vec2::splat(CELL_SIZE * 1.2)),
                ..default()
            },
            Transform::from_xyz(
                ant.pos.x as f32 * CELL_SIZE,
                -(ant.pos.y as f32 * CELL_SIZE),
                3.0,
            ),
            AntSprite { ant_id: ant.id },
        ));
    }
}

pub fn update_ant_sprites(
    mut query: Query<(&AntSprite, &mut Sprite, &mut Transform)>,
    simulation_state: Res<SimulationState>,
    pixel_assets: Res<PixelAssets>,
) {
    let snapshot = match &simulation_state.snapshot {
        Some(s) => s,
        None => return,
    };

    for (ant_sprite, mut sprite, mut transform) in query.iter_mut() {
        if let Some(ant) = snapshot.ants.iter().find(|a| a.id == ant_sprite.ant_id) {
            // Update direction-based sprite
            sprite.image = ant_sprite_handle(&pixel_assets, ant.direction);
            sprite.color = agitation_tint(ant.agitation);

            if ant.stress > 0.7 {
                let pulse = (ant.agitation * 0.3).min(0.3);
                sprite.color = sprite.color.mix(&Color::srgb(1.0, 0.3, 0.3), pulse);
            }

            transform.translation.x = ant.pos.x as f32 * CELL_SIZE;
            transform.translation.y = -(ant.pos.y as f32 * CELL_SIZE);

            // Carrying indicator: slightly larger
            if ant.carrying.is_some() {
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE * 1.4));
            } else {
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE * 1.2));
            }
        }
    }
}

fn agitation_tint(agitation: f32) -> Color {
    let calm = Color::srgb(1.0, 0.95, 0.85);
    let active = Color::srgb(1.0, 0.8, 0.6);
    calm.mix(&active, agitation)
}
