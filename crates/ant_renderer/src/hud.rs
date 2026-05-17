use bevy::prelude::*;
use ant_simulation::tick::{Simulation, Speed};

#[derive(Resource)]
pub struct HudState {
    pub panel_visible: bool,
}

impl Default for HudState {
    fn default() -> Self {
        Self { panel_visible: false }
    }
}

#[derive(Component)]
pub struct BottomBar;

#[derive(Component)]
pub struct TickText;

#[derive(Component)]
pub struct SpeedText;

#[derive(Component)]
pub struct QueenStatusText;

#[derive(Component)]
pub struct WorkerCountText;

#[derive(Component)]
pub struct SidePanel;

pub fn setup_hud(mut commands: Commands) {
    commands.spawn((
        BottomBar,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(32.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        parent.spawn((
            Button,
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        )).with_children(|btn| {
            btn.spawn(Text::new("||"));
        });

        parent.spawn((
            Button,
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        )).with_children(|btn| {
            btn.spawn(Text::new(">"));
        });

        parent.spawn((
            Button,
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        )).with_children(|btn| {
            btn.spawn(Text::new(">>"));
        });

        parent.spawn(Node {
            width: Val::Px(2.0),
            height: Val::Px(20.0),
            ..default()
        }).insert(BackgroundColor(Color::srgb(0.3, 0.3, 0.3)));

        parent.spawn((
            TickText,
            Text::new("Tick: 0"),
            Node { margin: UiRect::horizontal(Val::Px(12.0)), ..default() },
        ));

        parent.spawn((
            SpeedText,
            Text::new("1x"),
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        ));

        parent.spawn((
            QueenStatusText,
            Text::new("Queen: ●"),
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        ));

        parent.spawn((
            WorkerCountText,
            Text::new("Workers: 0"),
            Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
        ));
    });

    commands.spawn((
        SidePanel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            top: Val::Px(40.0),
            width: Val::Px(220.0),
            height: Val::Px(400.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
    )).with_children(|parent| {
        parent.spawn(Text::new("Queen: \u{25cf} (decorative only)\n\nEggs: 0\nLarvae: 0\nWorkers: 0\n\nAvg Humidity: 30%\nTemperature: 22.0\u{00b0}C\nTunnels dug: 0\nCollapses: 0"));
    });
}

pub fn update_hud(
    simulation: Res<Simulation>,
    mut tick_text: Query<&mut Text, (With<TickText>, Without<SpeedText>)>,
    mut speed_text: Query<&mut Text, (With<SpeedText>, Without<TickText>)>,
    mut queen_text: Query<&mut Text, (With<QueenStatusText>, Without<TickText>, Without<SpeedText>)>,
    mut worker_text: Query<&mut Text, (With<WorkerCountText>, Without<TickText>, Without<SpeedText>, Without<QueenStatusText>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel: Query<&mut Node, With<SidePanel>>,
    mut hud_state: ResMut<HudState>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        hud_state.panel_visible = !hud_state.panel_visible;
        for mut node in panel.iter_mut() {
            node.display = if hud_state.panel_visible { Display::Flex } else { Display::None };
        }
    }

    for mut text in tick_text.iter_mut() {
        text.0 = format!("Tick: {}  |  Day: {}", simulation.tick, simulation.day());
    }

    for mut text in speed_text.iter_mut() {
        let label = match simulation.speed {
            Speed::Paused => "||",
            Speed::Normal => "1x",
            Speed::Fast => "2x",
            Speed::Fastest => "4x",
        };
        text.0 = label.to_string();
    }

    for mut text in queen_text.iter_mut() {
        text.0 = "Queen: \u{25cf}".to_string();
    }

    for mut text in worker_text.iter_mut() {
        text.0 = "Workers: 0  |  Food: 0".to_string();
    }
}
