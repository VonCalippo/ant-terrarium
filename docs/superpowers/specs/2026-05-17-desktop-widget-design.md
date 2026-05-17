# Desktop Widget — Design Spec

**Date**: 2026-05-17
**Status**: Approved
**Part of**: Ant Terrarium Simulator (Sub-project 6 of 6)

## Overview

Transform the window into a compact, frameless desktop widget with always-on-top toggle.

## Features

### Frameless window
- No title bar, no borders
- Window chrome: only the terrarium glass border + HUD
- Draggable via click-drag on the bottom HUD bar

### Always-on-top toggle
- Default: ON (window stays above other apps)
- Toggle with F12 key
- Visual indicator in HUD: "📌" when pinned

### Compact mode
- Default window size: 800×600 (smaller than current 1024×768)
- Resizable as before (corner drag works even frameless)
- Minimum size: 400×300

### HUD drag area
- Bottom bar serves as drag handle
- Click on bottom bar (not buttons) + drag moves window
- Right-click on HUD bar shows context menu (future)

## Implementation

In `main.rs`, window config:
```rust
Window {
    title: "Ant Terrarium".into(),
    resolution: (800.0, 600.0).into(),
    resizable: true,
    decorations: false,          // frameless
    transparent: false,
    window_level: WindowLevel::AlwaysOnTop,
    ..default()
}
```

HUD bar gets a drag system that calls `window.set_position()` on drag.

## Out of Scope
- System tray
- Auto-start with Windows
- Transparency effects
