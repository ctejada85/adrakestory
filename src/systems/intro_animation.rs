use bevy::prelude::*;
use crate::components::{IntroUI, IntroText};
use crate::resources::{IntroAnimationTimer, IntroPhase};
use crate::states::GameState;

pub fn setup_intro(mut commands: Commands) {
    // Insert the animation timer resource
    commands.insert_resource(IntroAnimationTimer::new());

    // Root UI node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
            IntroUI,
        ))
        .with_children(|parent| {
            // Intro text - centered
            parent.spawn((
                Text::new("Presented by Kibound"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 0.9, 0.0)),
                IntroText,
            ));
        });
}

pub fn animate_intro(
    time: Res<Time>,
    mut timer: ResMut<IntroAnimationTimer>,
    mut text_query: Query<&mut TextColor, With<IntroText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.timer.tick(time.delta());

    if let Ok(mut text_color) = text_query.get_single_mut() {
        match timer.phase {
            IntroPhase::FadeIn => {
                // Fade in over 200ms
                let alpha = timer.timer.fraction();
                text_color.0.set_alpha(alpha);

                if timer.timer.just_finished() {
                    timer.phase = IntroPhase::Display;
                    timer.timer = Timer::from_seconds(1.5, TimerMode::Once);
                }
            }
            IntroPhase::Display => {
                // Display text fully visible
                text_color.0.set_alpha(1.0);

                if timer.timer.just_finished() {
                    timer.phase = IntroPhase::FadeOut;
                    timer.timer = Timer::from_seconds(0.2, TimerMode::Once);
                }
            }
            IntroPhase::FadeOut => {
                // Fade out over 200ms
                let alpha = 1.0 - timer.timer.fraction();
                text_color.0.set_alpha(alpha);

                if timer.timer.just_finished() {
                    // Transition to title screen
                    next_state.set(GameState::TitleScreen);
                }
            }
        }
    }
}

pub fn cleanup_intro(
    mut commands: Commands,
    query: Query<Entity, With<IntroUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<IntroAnimationTimer>();
}