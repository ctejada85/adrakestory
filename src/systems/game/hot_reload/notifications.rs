//! Reload notification UI components and systems.

use super::MapReloadedEvent;
use bevy::prelude::*;

/// Component for reload notification UI displayed in-game
#[derive(Component)]
pub struct ReloadNotification {
    /// Time when the notification was spawned (elapsed seconds)
    pub spawn_time: f32,
    /// Duration in seconds before the notification is despawned
    pub duration: f32,
}

/// System to show a notification when map reload completes
/// Spawns a text notification that fades out over time
pub fn show_reload_notification(
    mut commands: Commands,
    mut reloaded_events: EventReader<MapReloadedEvent>,
    time: Res<Time>,
) {
    for event in reloaded_events.read() {
        let color = if event.success {
            Color::srgba(0.2, 0.9, 0.2, 1.0) // Green for success
        } else {
            Color::srgba(0.9, 0.2, 0.2, 1.0) // Red for failure
        };

        info!("Showing reload notification: {}", event.message);

        // Spawn notification text with UI components
        commands.spawn((
            Text::new(&event.message),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(color),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Percent(50.0),
                // Center the text horizontally
                justify_self: JustifySelf::Center,
                ..default()
            },
            ReloadNotification {
                spawn_time: time.elapsed_secs(),
                duration: 2.5,
            },
        ));
    }
}

/// System to fade out and despawn reload notifications
/// Notifications fade out during the last 30% of their duration
pub fn update_reload_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ReloadNotification, &mut TextColor)>,
) {
    for (entity, notification, mut text_color) in query.iter_mut() {
        let elapsed = time.elapsed_secs() - notification.spawn_time;

        if elapsed > notification.duration {
            // Notification expired, despawn it
            commands.entity(entity).despawn();
        } else if elapsed > notification.duration * 0.7 {
            // Fade out during last 30% of duration
            let fade_progress =
                (elapsed - notification.duration * 0.7) / (notification.duration * 0.3);
            let alpha = 1.0 - fade_progress;
            text_color.0 = text_color.0.with_alpha(alpha);
        }
    }
}
