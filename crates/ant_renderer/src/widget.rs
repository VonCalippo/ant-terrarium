use bevy::{prelude::*, window::WindowLevel};

pub fn window_drag(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    hud_bar: Query<&Node, With<super::hud::BottomBar>>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }

    let Ok(mut window) = windows.get_single_mut() else { return };
    let Ok(hud_node) = hud_bar.get_single() else { return };

    // Check if mouse is in the bottom bar area for drag
    if let Some(cursor) = window.cursor_position() {
        let bar_height = match hud_node.height {
            Val::Px(h) => h,
            _ => 32.0,
        };
        let bar_top = window.height() - bar_height;
        if cursor.y > bar_top {
            window.start_drag_move();
        }
    }
}

pub fn toggle_always_on_top(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    mut pinned: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        *pinned = !*pinned;
        if let Ok(mut window) = windows.get_single_mut() {
            window.window_level = if *pinned {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            };
        }
    }
}

pub fn escape_close(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
