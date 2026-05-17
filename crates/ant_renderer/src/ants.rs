use bevy::prelude::*;
use crate::app::SimResource;
use crate::sprites::SimulationState;
use crate::assets::{self, CELL_SIZE};

#[derive(Component)]
pub struct AntSprite {
    pub ant_id: usize,
}

pub fn spawn_ant_sprites(
    mut commands: Commands,
    simulation: Res<SimResource>,
) {
    let snap = ant_simulation::snapshot::Snapshot::from_simulation(&simulation);

    for ant in &snap.ants {
        let color = assets::ant_body_color(ant.agitation, ant.carrying);
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(CELL_SIZE * 0.7)),
                ..default()
            },
            Transform::from_xyz(
                ant.pos.x as f32 * CELL_SIZE,
                -(ant.pos.y as f32 * CELL_SIZE),
                2.0,
            ),
            AntSprite { ant_id: ant.id },
        ));
    }
}

pub fn update_ant_sprites(
    mut query: Query<(&AntSprite, &mut Sprite, &mut Transform)>,
    simulation_state: Res<SimulationState>,
) {
    let snapshot = match &simulation_state.snapshot {
        Some(s) => s,
        None => return,
    };

    for (ant_sprite, mut sprite, mut transform) in query.iter_mut() {
        if let Some(ant) = snapshot.ants.iter().find(|a| a.id == ant_sprite.ant_id) {
            let color = assets::ant_body_color(ant.agitation, ant.carrying);

            // Slightly pulse toward stress color
            if ant.stress > 0.7 {
                let pulse = (ant.agitation * 0.3).min(0.3);
                sprite.color = color.mix(&Color::srgb(0.9, 0.2, 0.2), pulse);
            } else {
                sprite.color = color;
            }

            transform.translation.x = ant.pos.x as f32 * CELL_SIZE;
            transform.translation.y = -(ant.pos.y as f32 * CELL_SIZE);
        }
    }
}
