use std::env;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{close_on_esc, WindowMode},
};
use bevy_pancam::{PanCam, PanCamPlugin};

use island_procgen::{minigame::MinigamePlugin, player::PlayerPlugin, terrain::TerrainPlugin};
use island_procgen::{terrain::ResetTerrainEvent, *};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut mode = WindowMode::default();
    if !args.contains(&ARG_DISABLE_FULLSCREEN.to_string()) {
        mode = bevy::window::WindowMode::Fullscreen;
    }

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode,
                        resolution: (WW as f32, WH as f32).into(),
                        title: "ProcGen".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 0,
        )))
        .add_plugins(PanCamPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(TerrainPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MinigamePlugin)
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, handle_settings_input)
        .add_systems(Update, close_on_esc)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());
}

fn handle_settings_input(keys: Res<Input<KeyCode>>, mut writer: EventWriter<ResetTerrainEvent>) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    writer.send(ResetTerrainEvent);
}
