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

pub(crate) fn spawn_ant_sprites(
    mut commands: Commands,
    simulation: Res<SimResource>,
    pixel_assets: Res<PixelAssets>,
) {
    let snap = ant_simulation::snapshot::Snapshot::from_simulation(&simulation);

    for ant in &snap.ants {
        let image = ant_sprite_handle(&pixel_assets, ant.direction);
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

            sprite.image = ant_sprite_handle(&pixel_assets, ant.direction);

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
