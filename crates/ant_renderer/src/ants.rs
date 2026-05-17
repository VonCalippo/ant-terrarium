use bevy::prelude::*;
use crate::app::SimResource;
use crate::sprites::SimulationState;
use crate::pixelart::{PixelAssets, ant_sprite_handle};
use crate::assets::CELL_SIZE;

#[derive(Component)]
pub struct AntSprite {
    pub ant_id: usize,
    pub target_pos: Vec2,
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
        let pos = vec2(ant.pos.x as f32 * CELL_SIZE, -(ant.pos.y as f32 * CELL_SIZE));

        commands.spawn((
            Sprite {
                image,
                color: tint,
                custom_size: Some(Vec2::splat(CELL_SIZE * 1.2)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 3.0),
            AntSprite { ant_id: ant.id, target_pos: pos },
        ));
    }
}

pub fn update_ant_sprites(
    time: Res<Time>,
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

            // Smooth movement: lerp toward target
            let target = vec2(ant.pos.x as f32 * CELL_SIZE, -(ant.pos.y as f32 * CELL_SIZE));
            let speed = 12.0; // lerp speed
            let current = vec2(transform.translation.x, transform.translation.y);
            let new_pos = current.lerp(target, (time.delta_secs() * speed).min(1.0));
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;

            // Quick snap if very close
            if current.distance_squared(target) < 0.01 {
                transform.translation.x = target.x;
                transform.translation.y = target.y;
            }
        }
    }
}

fn agitation_tint(agitation: f32) -> Color {
    let calm = Color::srgb(1.0, 0.95, 0.85);
    let active = Color::srgb(1.0, 0.8, 0.6);
    calm.mix(&active, agitation)
}

fn vec2(x: f32, y: f32) -> Vec2 { Vec2::new(x, y) }
