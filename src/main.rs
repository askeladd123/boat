#![allow(unused)]

use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    log::LogPlugin,
    prelude::{default, *},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier3d::prelude::*;
use std::{collections::HashMap, rc::Rc};

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
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
        ))
        .add_plugins(EguiPlugin)
        .add_state::<AssetState>()
        .add_systems(Startup, spawn_entities)
        .add_systems(PostStartup, start_loading_assets)
        .add_systems(
            Update,
            check_if_vital_assets_loaded.run_if(in_state(AssetState::Loading)),
        )
        .add_systems(Update, update_ui)
        .add_systems(OnEnter(AssetState::Loaded), add_vital_assets)
        .add_systems(
            Update,
            (keyboard_input_system, move_camera, add_env_forces)
                .run_if(in_state(AssetState::Loaded)),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AssetState {
    #[default]
    Loading,
    Loaded,
    Failed,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Land;

#[derive(Component)]
struct MovingObject;

#[derive(Component)]
struct Camera;

#[derive(Component)]
struct LandCollider;

#[derive(Resource)]
struct AssetsVital {
    bbox: HashMap<String, Handle<Gltf>>,
}

fn spawn_entities(mut cmd: Commands) {
    cmd.spawn((
        Player,
        MovingObject,
        RigidBody::Dynamic,
        Velocity {
            linvel: Vec3 {
                x: 0.,
                y: 10.,
                z: 0.,
            },
            ..default()
        },
    ));
    cmd.spawn(Camera);
}

fn start_loading_assets(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    player: Query<Entity, With<Player>>,
    camera: Query<Entity, With<Camera>>,
) {
    // these assets are vital, and the rest of the program need to wait for them
    cmd.insert_resource(AssetsVital {
        bbox: HashMap::from([
            ("boats".to_string(), asset_server.load("boats-bbox.glb")),
            ("islands".to_string(), asset_server.load("islands-bbox.glb")),
        ]),
    });

    // these assets are not vital, so the rest of the program does not need to wait for them
    cmd.entity(camera.single()).insert(SceneBundle {
        scene: asset_server.load("cameras.glb#Scene0"),
        ..default()
    });
    cmd.spawn(SceneBundle {
        scene: asset_server.load("lights.glb#Scene0"),
        ..default()
    });
    cmd.entity(player.single()).insert(SceneBundle {
        scene: asset_server.load("boats.glb#Scene0"),
        ..default()
    });
    cmd.spawn(SceneBundle {
        scene: asset_server.load("islands.glb#Scene0"),
        ..default()
    });
    cmd.spawn(SceneBundle {
        scene: asset_server.load("ocean.glb#Scene0"),
        ..default()
    });
}

fn check_if_vital_assets_loaded(
    asset_server: Res<AssetServer>,
    handles: Res<AssetsVital>,
    mut next_state: ResMut<NextState<AssetState>>,
) {
    match asset_server.get_group_load_state(handles.bbox.iter().map(|kv| kv.1.id())) {
        bevy::asset::LoadState::Loaded => next_state.set(AssetState::Loaded),
        bevy::asset::LoadState::Failed => next_state.set(AssetState::Failed),
        _ => {}
    }
}

fn add_vital_assets(
    mut cmd: Commands,
    handles: Res<AssetsVital>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltf_nodes: Res<Assets<GltfNode>>,
    assets_gltf_mesh: Res<Assets<GltfMesh>>,
    assets_mesh: Res<Assets<Mesh>>,
    player: Query<Entity, With<Player>>,
) {
    let mesh = assets_mesh
        .get(
            &assets_gltf_mesh
                .get(&assets_gltf.get(&handles.bbox["boats"]).unwrap().meshes[0])
                .unwrap()
                .primitives[0]
                .mesh,
        )
        .unwrap();

    cmd.entity(player.single())
        .insert(Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::ConvexHull).unwrap());

    let gltf = assets_gltf.get(&handles.bbox["islands"]).unwrap();
    for node in gltf
        .nodes
        .iter()
        .map(|node_handle| assets_gltf_nodes.get(node_handle).unwrap())
    {
        let mesh = assets_mesh
            .get(
                &assets_gltf_mesh
                    .get(&node.mesh.clone().unwrap())
                    .unwrap()
                    .primitives[0]
                    .mesh,
            )
            .unwrap();
        let transform = node.transform;
        cmd.spawn((
            RigidBody::Fixed,
            Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap(),
            LandCollider,
            TransformBundle::from(transform),
        ));
    }
}

fn update_ui(state: Res<State<AssetState>>, mut contexts: EguiContexts) {
    egui::Window::new("debug control panel").show(contexts.ctx_mut(), |ui| match state.get() {
        AssetState::Loading => ui.label("loading assets for vital functions"),
        AssetState::Loaded => ui.label("press arrow keys to move the boat"),
        AssetState::Failed => {
            ui.label("assets failed to load for some reason, check console for detailed errors")
        }
    });
}

fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let mut camera = camera.single_mut();
    camera.translation.x = player.single().translation.x;
    camera.translation.z = player.single().translation.z;
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity), With<Player>>,
) {
    const TURN_SPEED: f32 = 3.;
    const SPEED: f32 = 5.;

    let (trans, mut vel) = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        vel.angvel = [0., TURN_SPEED, 0.].into();
    }
    if keyboard_input.pressed(KeyCode::Right) {
        vel.angvel = [0., -TURN_SPEED, 0.].into();
    }
    if keyboard_input.pressed(KeyCode::Up) {
        let forward = trans.forward();
        vel.linvel = forward * SPEED;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        let forward = trans.back();
        vel.linvel = forward * SPEED * 0.66;
    }
}

fn add_env_forces(
    mut floating_objects: Query<(&mut Transform, &mut Velocity), With<MovingObject>>,
) {
    const AVG_BOAT_HEIGHT: f32 = 1.;
    const FLOAT_C: f32 = 1.;
    const DRAG_C: f32 = 0.05;
    const DRAG_ANG_C: f32 = 0.05;

    //TODO: Transform trenger ikke være mut her
    for (trans, mut vel) in floating_objects.iter_mut() {
        // # bouancy from water
        //TODO: do this in a continuous way instead, without if statements; just for fun and practice ofcousrse
        let y = trans.translation.y;
        let mut v = 0.;
        if y < 0. {
            if -y > AVG_BOAT_HEIGHT {
                v = AVG_BOAT_HEIGHT * FLOAT_C;
            } else {
                v = -y * FLOAT_C;
            }
        }
        vel.linvel.y += v;
        let inverse = -vel.linvel;
        vel.linvel += inverse * DRAG_C;

        // # drag from turning and moving forward
        let speed = vel.linvel.length();
        if 0.001 < speed {
            let normal = vel.linvel.normalize();
            vel.linvel -= normal * DRAG_C * speed;
        }

        let speed = vel.angvel.length();
        if 0.001 < speed {
            let normal = vel.angvel.normalize();
            vel.angvel -= normal * DRAG_ANG_C * speed
        }
    }
}
