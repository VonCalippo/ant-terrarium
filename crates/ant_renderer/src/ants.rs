use bevy::prelude::*;
use crate::app::SimResource;
use crate::sprites::SimulationState;
use crate::pixelart::{PixelAssets, ant_sprite_handle};
use crate::assets::CELL_SIZE;

#[derive(Component)]
pub struct AntSprite {
    pub ant_id: usize,
}

pub(crate) fn spawn_ant_sprites(
    mut commands: Commands,
    simulation: Res<SimResource>,
    pixel_assets: Res<PixelAssets>,
) {
    let snap = ant_simulation::snapshot::Snapshot::from_simulation(&simulation);

    for ant in &snap.ants {
        let image = ant_sprite_handle(&pixel_assets, ant.direction);
        let tint = agitation_tint(ant.agitation);
        let x = ant.pos.x as f32 * CELL_SIZE;
        let y = -(ant.pos.y as f32 * CELL_SIZE);

        commands.spawn((
            Sprite {
                image,
                color: tint,
                custom_size: Some(Vec2::splat(CELL_SIZE * 1.2)),
                ..default()
            },
            Transform::from_xyz(x, y, 3.0),
            AntSprite { ant_id: ant.id },
            AntTarget { x, y },
        ));
    }
}

#[derive(Component, Clone, Copy)]
pub(crate) struct AntTarget { pub(crate) x: f32, pub(crate) y: f32 }

pub(crate) fn update_ant_sprites(
    time: Res<Time>,
    mut query: Query<(&AntSprite, &mut Sprite, &mut Transform, &mut AntTarget)>,
    simulation_state: Res<SimulationState>,
    pixel_assets: Res<PixelAssets>,
) {
    let snapshot = match &simulation_state.snapshot {
        Some(s) => s,
        None => return,
    };

    let dt = time.delta_secs();
    let lerp_speed = 15.0;

    for (ant_sprite, mut sprite, mut transform, mut target) in query.iter_mut() {
        if let Some(ant) = snapshot.ants.iter().find(|a| a.id == ant_sprite.ant_id) {
            // Update target position from sim
            target.x = ant.pos.x as f32 * CELL_SIZE;
            target.y = -(ant.pos.y as f32 * CELL_SIZE);

            // Direction sprite
            sprite.image = ant_sprite_handle(&pixel_assets, ant.direction);

            // Color tint
            let mut tint = agitation_tint(ant.agitation);
            if ant.stress > 0.7 {
                let pulse = (ant.agitation * 0.3).min(0.3);
                tint = tint.mix(&Color::srgb(1.0, 0.3, 0.3), pulse);
            }
            sprite.color = tint;

            // Size
            if ant.carrying.is_some() {
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE * 1.4));
            } else {
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE * 1.2));
            }

            // Smooth lerp toward target every frame
            let t = (dt * lerp_speed).min(1.0);
            transform.translation.x += (target.x - transform.translation.x) * t;
            transform.translation.y += (target.y - transform.translation.y) * t;
        }
    }
}

fn agitation_tint(agitation: f32) -> Color {
    let calm = Color::srgb(1.0, 0.95, 0.85);
    let active = Color::srgb(1.0, 0.8, 0.6);
    calm.mix(&active, agitation)
}
