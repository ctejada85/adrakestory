use bevy::{prelude::*, window::WindowMode};
use std::path::PathBuf;

pub mod editor;
mod states;
mod systems;

use states::GameState;
use systems::game::gamepad::{
    gather_gamepad_input, gather_keyboard_input, handle_gamepad_connections, reset_player_input,
    update_cursor_visibility, ActiveGamepad, GamepadSettings, PlayerInput,
};

/// Command-line arguments for the game
#[derive(Debug, Default)]
struct GameArgs {
    /// Path to map file to load directly (skips intro and title screen)
    map_path: Option<PathBuf>,
}

/// Resource to hold command-line specified map path for direct loading
#[derive(Resource, Default)]
pub struct CommandLineMapPath {
    pub path: Option<PathBuf>,
}

/// Parse command-line arguments
fn parse_args() -> GameArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut game_args = GameArgs::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--map" | "-m" => {
                if i + 1 < args.len() {
                    game_args.map_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                } else {
                    eprintln!("Warning: --map requires a path argument");
                }
            }
            "--help" | "-h" => {
                println!("A Drake's Story");
                println!();
                println!("Usage: adrakestory [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -m, --map <PATH>  Load a specific map file directly (skips title screen)");
                println!("  -h, --help        Show this help message");
                std::process::exit(0);
            }
            _ => {
                // Ignore unknown arguments
            }
        }
        i += 1;
    }

    game_args
}

/// System sets for organizing game loop execution order.
/// These sets ensure proper sequencing of game logic phases.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSystemSet {
    /// Handle user input (keyboard, mouse, gamepad)
    Input,
    /// Process player movement based on input
    Movement,
    /// Apply physics simulation (gravity, collisions)
    Physics,
    /// Update visual elements (collision box, effects, etc.)
    Visual,
    /// Update camera position and rotation
    Camera,
}
use systems::game::map::{
    spawn_map_system, update_chunk_lods, LoadedMapData, MapLoadProgress, MapLoader,
};
use systems::game::systems::{
    apply_gravity, apply_npc_collision, apply_physics, follow_player_camera, handle_escape_key,
    move_player, rotate_camera, rotate_character_model, toggle_collision_box, toggle_fullscreen,
    update_collision_box,
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
    // Parse command-line arguments
    let args = parse_args();
    let has_map_arg = args.map_path.is_some();

    // Determine initial state based on CLI arguments
    // If a map is specified, skip intro and title screen
    let initial_state = if has_map_arg {
        GameState::LoadingMap
    } else {
        GameState::IntroAnimation
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                ..default()
            }),
            ..default()
        }))
        .insert_state(initial_state)
        .insert_resource(CommandLineMapPath { path: args.map_path })
        .init_resource::<MapLoadProgress>()
        // Initialize gamepad resources
        .init_resource::<ActiveGamepad>()
        .init_resource::<GamepadSettings>()
        .init_resource::<PlayerInput>()
        .add_systems(Startup, setup)
        // Global systems that run in any state
        .add_systems(Update, (toggle_fullscreen, handle_gamepad_connections))
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
        .add_systems(
            OnEnter(GameState::InGame),
            (cleanup_2d_camera, spawn_map_system).chain(),
        )
        // Configure system sets to run in a specific order
        .configure_sets(
            Update,
            (
                GameSystemSet::Input,
                GameSystemSet::Movement,
                GameSystemSet::Physics,
                GameSystemSet::Visual,
                GameSystemSet::Camera,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        // Input phase: Gather input from all sources, then handle game-specific input
        .add_systems(
            Update,
            (
                reset_player_input,
                gather_gamepad_input,
                gather_keyboard_input,
                handle_escape_key,
                toggle_collision_box,
            )
                .chain()
                .in_set(GameSystemSet::Input),
        )
        // Movement phase: Process player movement
        .add_systems(Update, move_player.in_set(GameSystemSet::Movement))
        // Physics phase: Apply gravity and physics (in order)
        .add_systems(
            Update,
            (apply_gravity, apply_physics, apply_npc_collision)
                .chain()
                .in_set(GameSystemSet::Physics),
        )
        // Visual phase: Update visual elements after all position changes
        .add_systems(
            Update,
            (
                rotate_character_model,
                update_collision_box,
                update_chunk_lods,
                update_cursor_visibility,
            )
                .in_set(GameSystemSet::Visual),
        )
        // Camera phase: Update camera last (follow then rotate)
        .add_systems(
            Update,
            (follow_player_camera, rotate_camera)
                .chain()
                .in_set(GameSystemSet::Camera),
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
    // 2D camera for UI elements (title screen, loading screen, etc.)
    commands.spawn(Camera2d);
}

/// System to despawn the 2D camera when entering InGame state.
/// This prevents camera order ambiguity with the 3D game camera.
fn cleanup_2d_camera(mut commands: Commands, camera_query: Query<Entity, With<Camera2d>>) {
    for entity in &camera_query {
        commands.entity(entity).despawn();
        info!("Despawned 2D camera before entering InGame state");
    }
}

/// System to load the map when entering LoadingMap state.
fn load_map_on_enter(
    mut commands: Commands,
    mut progress: ResMut<MapLoadProgress>,
    cli_map_path: Res<CommandLineMapPath>,
) {
    info!("Loading map...");
    progress.clear();

    // Determine which map file to load
    // Priority: CLI argument > default map
    let map_path = if let Some(path) = &cli_map_path.path {
        info!("Loading map from command-line argument: {:?}", path);
        path.to_string_lossy().to_string()
    } else {
        "assets/maps/default.ron".to_string()
    };

    // Try to load the specified map file
    let map = match MapLoader::load_from_file(&map_path, &mut progress) {
        Ok(map) => {
            info!("Successfully loaded map: {}", map.metadata.name);
            map
        }
        Err(e) => {
            warn!("Failed to load map file '{}': {}. Using default map.", map_path, e);
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
