#![allow(unused)]

use bevy::{
    gltf::Gltf,
    log::LogPlugin,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::{f32::consts::PI, sync::Arc};

const CAMERA_OFFSET: Vec3 = Vec3 {
    x: 0.,
    y: 12.,
    z: 6.,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    filter: "warn,seilespill=trace".into(),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin)
        .add_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Startup, load_assets)
        .add_systems(Update, update_ui)
        .add_systems(Update, check_if_loaded.run_if(in_state(AppState::Loading)))
        .add_systems(OnEnter(AppState::Loaded), add_assets)
        .add_systems(
            Update,
            (keyboard_input_system, move_camera).run_if(in_state(AppState::Loaded)),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Loaded,
    Failed,
}

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct BlenderAssets(Handle<Gltf>);

const X_EXTENT: f32 = 14.5;

fn check_if_loaded(
    mut next_state: ResMut<NextState<AppState>>,
    assets: Res<BlenderAssets>,
    asset_loader: Res<AssetServer>,
) {
    match asset_loader.get_load_state(&assets.0) {
        bevy::asset::LoadState::Loaded => next_state.set(AppState::Loaded),
        bevy::asset::LoadState::Failed => next_state.set(AppState::Failed),
        _ => {}
    }
}

fn load_assets(mut cmd: Commands, mut asset_server: Res<AssetServer>) {
    cmd.insert_resource(BlenderAssets(asset_server.load("seilespill.glb")));
}

fn add_assets(
    mut cmd: Commands,
    mut assets: Res<BlenderAssets>,
    asset_loader: Res<AssetServer>,
    gltf_assets: Res<Assets<Gltf>>,
) {
    cmd.spawn(SceneBundle {
        scene: gltf_assets.get(&assets.0).unwrap().named_scenes["scene-boats"].clone(),
        // scene: gltf_assets.get(&assets.0).unwrap().scenes[0].clone(),
        ..default()
    })
    .insert(Player);

    cmd.spawn(SceneBundle {
        scene: gltf_assets.get(&assets.0).unwrap().named_scenes["scene-map"].clone(),
        // scene: gltf_assets.get(&assets.0).unwrap().scenes[1].clone(),
        ..default()
    });
}

#[derive(Component)]
struct MyCamera;

fn setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(CAMERA_OFFSET)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(MyCamera);
}

fn update_ui(state: Res<State<AppState>>, mut contexts: EguiContexts) {
    egui::Window::new("debug control panel").show(contexts.ctx_mut(), |ui| match state.get() {
        AppState::Loading => ui.label("loading beautiful graphics"),
        AppState::Loaded => ui.label("press arrow keys to move the boat"),
        AppState::Failed => {
            ui.label("assets failed to load for some reason, check console for detailed errors")
        }
    });
}

fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<MyCamera>, Without<Player>)>,
) {
    camera.single_mut().translation = player.single().translation + CAMERA_OFFSET;
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    const TURN_SPEED: f32 = 0.05;
    const SPEED: f32 = 0.05;

    let mut player_transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        player_transform.rotate_y(TURN_SPEED);
    }
    if keyboard_input.pressed(KeyCode::Right) {
        player_transform.rotate_y(-TURN_SPEED);
    }
    if keyboard_input.pressed(KeyCode::Up) {
        let forward = player_transform.forward();
        player_transform.translation += forward * SPEED;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        let forward = player_transform.forward();
        player_transform.translation += forward * -SPEED;
    }
}
