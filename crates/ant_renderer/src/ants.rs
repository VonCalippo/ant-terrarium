use bevy::prelude::*;
use ant_simulation::ant::Action;
use crate::app::SimResource;
use crate::sprites::SimulationState;
use crate::pixelart::{PixelAssets, ant_sprite_handle};
use crate::assets::CELL_SIZE;

const ANT_SIZE: f32 = 2.0;   // multiple of CELL_SIZE
const ANT_CARRY_SIZE: f32 = 2.4;

#[derive(Component)]
pub struct AntSprite {
    pub ant_id: usize,
}

#[derive(Component)]
pub struct Particle {
    pub life: f32,
    pub max_life: f32,
}

pub(crate) fn spawn_ant_sprites(
    mut commands: Commands,
    simulation: Res<SimResource>,
    pixel_assets: Res<PixelAssets>,
) {
    let snap = ant_simulation::snapshot::Snapshot::from_simulation(&simulation);

    for ant in &snap.ants {
        let image = ant_sprite_handle(&pixel_assets, ant.direction, 0);
        let x = ant.pos.x as f32 * CELL_SIZE;
        let y = -(ant.pos.y as f32 * CELL_SIZE);

        commands.spawn((
            Sprite {
                image,
                color: Color::WHITE,
                custom_size: Some(Vec2::splat(CELL_SIZE * ANT_SIZE)),
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
    let lerp_speed = 18.0;

    for (ant_sprite, mut sprite, mut transform, mut target) in query.iter_mut() {
        if let Some(ant) = snapshot.ants.iter().find(|a| a.id == ant_sprite.ant_id) {
            target.x = ant.pos.x as f32 * CELL_SIZE;
            target.y = -(ant.pos.y as f32 * CELL_SIZE);

            sprite.image = ant_sprite_handle(&pixel_assets, ant.direction, snapshot.tick);

            // Color: base body tint + action indicator
            sprite.color = action_tint(ant.action, ant.stress, ant.agitation);

            sprite.custom_size = Some(Vec2::splat(
                CELL_SIZE * if ant.carrying.is_some() { ANT_CARRY_SIZE } else { ANT_SIZE }
            ));

            let t = (dt * lerp_speed).min(1.0);
            transform.translation.x += (target.x - transform.translation.x) * t;
            transform.translation.y += (target.y - transform.translation.y) * t;
        }
    }
}

pub(crate) fn spawn_dig_particles(
    mut commands: Commands,
    query: Query<(&Transform, &AntSprite)>,
    simulation_state: Res<SimulationState>,
) {
    let snapshot = match &simulation_state.snapshot { Some(s) => s, None => return };
    for (transform, ant_sprite) in query.iter() {
        if let Some(ant) = snapshot.ants.iter().find(|a| a.id == ant_sprite.ant_id) {
            if matches!(ant.action, Action::Dig(_)) && snapshot.tick % 3 == 0 {
                let seed = (snapshot.tick + ant_sprite.ant_id as u64) * 12345;
                for i in 0..2 {
                    let ox = (quick_rand(seed + i as u64) - 0.5) * CELL_SIZE * 0.6;
                    let oy = (quick_rand(seed + i as u64 + 100) - 0.5) * CELL_SIZE * 0.6;
                    commands.spawn((
                        Sprite { color: Color::srgb(0.55, 0.40, 0.22), custom_size: Some(Vec2::splat(CELL_SIZE * 0.3)), ..default() },
                        Transform::from_xyz(transform.translation.x + ox, transform.translation.y + oy, 4.0),
                        Particle { life: 0.0, max_life: 0.3 + quick_rand(seed + i as u64 + 200) * 0.2 },
                    ));
                }
            }
        }
    }
}

pub(crate) fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut particle, mut sprite, mut transform) in query.iter_mut() {
        particle.life += time.delta_secs();
        let t = (particle.life / particle.max_life).min(1.0);
        sprite.color.set_alpha(1.0 - t);
        transform.translation.y += time.delta_secs() * 4.0; // float up
        if particle.life >= particle.max_life {
            commands.entity(entity).despawn();
        }
    }
}

fn quick_rand(seed: u64) -> f32 {
    let x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((x >> 32) as u32) as f32 / u32::MAX as f32
}

fn action_tint(action: Action, stress: f32, _agitation: f32) -> Color {
    let base = match action {
        Action::Idle | Action::Rest => Color::srgb(0.9, 0.85, 0.75),
        Action::Move(_) => Color::srgb(1.0, 0.9, 0.8),
        Action::Dig(_) => Color::srgb(0.9, 0.8, 0.3),       // yellow-ish when digging
        Action::CollectFood => Color::srgb(0.5, 1.0, 0.4),    // green when collecting
        Action::CarryFood { .. } => Color::srgb(0.6, 1.0, 0.4), // bright green carrying food
        Action::CarryDirt { .. } => Color::srgb(0.8, 0.6, 0.4), // brown carrying dirt
        Action::Eat => Color::srgb(0.5, 0.9, 0.5),
        Action::Flee { .. } => Color::srgb(1.0, 0.3, 0.2),    // red when fleeing
        Action::Groom => Color::srgb(0.7, 0.8, 1.0),
    };

    if stress > 0.7 {
        base.mix(&Color::srgb(1.0, 0.2, 0.2), (stress - 0.7) * 2.0)
    } else {
        base
    }
}
