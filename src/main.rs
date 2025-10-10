use bevy::prelude::*;

mod states;
mod systems;

use states::GameState;
use systems::game::map::{spawn_map_system, LoadedMapData, MapLoadProgress, MapLoader};
use systems::game::systems::{
    apply_gravity, apply_physics, handle_escape_key, move_player, rotate_camera,
    toggle_collision_box, update_collision_box,
};
use systems::intro_animation::systems::{animate_intro, cleanup_intro, setup_intro};
use systems::loading_screen::{
    cleanup_loading_screen, setup_loading_screen, update_loading_progress,
};
use systems::pause_menu::systems as pause_menu;
use systems::title_screen::systems::{
    button_interaction, cleanup_title_screen, fade_in_title_screen, keyboard_navigation,
    scale_text_on_resize, setup_title_screen, update_selected_button_visual,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_resource::<MapLoadProgress>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::IntroAnimation), setup_intro)
        .add_systems(
            Update,
            animate_intro.run_if(in_state(GameState::IntroAnimation)),
        )
        .add_systems(OnExit(GameState::IntroAnimation), cleanup_intro)
        .add_systems(OnEnter(GameState::TitleScreen), setup_title_screen)
        .add_systems(
            Update,
            (
                fade_in_title_screen,
                button_interaction,
                keyboard_navigation,
                update_selected_button_visual,
                scale_text_on_resize,
            )
                .run_if(in_state(GameState::TitleScreen)),
        )
        .add_systems(OnExit(GameState::TitleScreen), cleanup_title_screen)
        .add_systems(
            OnEnter(GameState::LoadingMap),
            (setup_loading_screen, load_map_on_enter),
        )
        .add_systems(
            Update,
            (update_loading_progress, check_map_loaded).run_if(in_state(GameState::LoadingMap)),
        )
        .add_systems(OnExit(GameState::LoadingMap), cleanup_loading_screen)
        .add_systems(OnEnter(GameState::InGame), spawn_map_system)
        .add_systems(
            Update,
            (
                move_player,
                rotate_camera,
                toggle_collision_box,
                update_collision_box,
                apply_gravity,
                apply_physics,
                handle_escape_key,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnEnter(GameState::Paused), pause_menu::setup_pause_menu)
        .add_systems(
            Update,
            (
                pause_menu::pause_menu_input,
                pause_menu::keyboard_navigation,
                pause_menu::pause_menu_button_interaction,
                pause_menu::update_selected_button_visual,
                pause_menu::scale_text_on_resize,
            )
                .run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnExit(GameState::Paused), pause_menu::cleanup_pause_menu)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// System to load the map when entering LoadingMap state.
fn load_map_on_enter(mut commands: Commands, mut progress: ResMut<MapLoadProgress>) {
    info!("Loading map...");
    progress.clear();

    // Try to load the default map file
    let map = match MapLoader::load_from_file("assets/maps/default.ron", &mut progress) {
        Ok(map) => {
            info!("Successfully loaded map: {}", map.metadata.name);
            map
        }
        Err(e) => {
            warn!("Failed to load map file: {}. Using default map.", e);
            progress.update(systems::game::map::LoadProgress::Error(e.to_string()));
            MapLoader::load_default()
        }
    };

    commands.insert_resource(LoadedMapData { map });
}

/// System to check if map loading is complete and transition to InGame state.
fn check_map_loaded(
    progress: Res<MapLoadProgress>,
    map_data: Option<Res<LoadedMapData>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // If we have map data and loading is complete (or errored but we have fallback)
    if map_data.is_some() && (progress.is_complete() || progress.percentage() >= 0.6) {
        info!("Map loading complete, transitioning to InGame");
        next_state.set(GameState::InGame);
    }
}
