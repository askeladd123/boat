#![allow(unused)]

use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    log::LogPlugin,
    pbr::DirectionalLightShadowMap,
    prelude::{default, *},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier3d::prelude::*;
use custom_assets::*;
use dock::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, iter::once};
use utils::*;

mod custom_assets;
mod dock;
mod utils;

fn main() {
    App::new()
        .add_systems(Update, (dock_menu_2).run_if(in_state(AssetState::Loaded)))
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    filter: "warn,wgpu_hal::vulkan::instance=off,seilespill=trace".into(),
                    ..default()
                }),
            JsonAssetPlugin::<ConfigValues>::new(&["json"]),
        ))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default().disabled(),
        ))
        .add_plugins(EguiPlugin)
        .add_state::<AssetState>()
        .add_event::<ConfigSave>()
        .add_event::<ConfigValuesChanged>()
        .add_event::<Dock>()
        .add_systems(Startup, spawn_entities)
        .add_systems(
            PostStartup,
            (
                start_loading_assets,
                // check_load_state,
                // load_config
            ),
        )
        .add_systems(
            Update,
            (check_load_state, check_if_vital_assets_loaded).run_if(in_state(AssetState::Loading)),
        )
        .add_systems(Update, (update_ui, save_config))
        .add_systems(
            OnEnter(AssetState::Loaded),
            (on_loaded_general, add_vital_assets),
        )
        .add_systems(
            Update,
            (
                keyboard_input_system,
                move_camera,
                add_env_forces,
                update_values,
                wire_sensor_events,
                wire_dock_events,
            )
                .run_if(in_state(AssetState::Loaded)),
        )
        .run();
}

//TODO: combine ConfigSave and ConfigValuesChanged into one enum to clean up

/// fires when the config needs to save
#[derive(Event)]
struct ConfigSave;

/// fires when the config is changed
#[derive(Event)]
struct ConfigValuesChanged;

#[derive(Event)]
enum Dock {
    Docking,
    UnDocking,
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
struct MovingObject;

#[derive(Component)]
struct Camera;

#[derive(Component)]
struct DockMenu;

#[derive(Resource)]
struct AssetPool {
    bboxes: Handle<Gltf>,
    font: Handle<Font>,
    config: Handle<ConfigValues>,
}

#[derive(Resource)]
struct AssetsVital {
    bboxes: Handle<Gltf>,
}

#[derive(Resource)]
struct AssetsNonvital {
    font: Handle<Font>,
}

#[derive(Default, Debug)]
enum DockState {
    #[default]
    TooFar,
    CloseTo(Entity),
    DockedTo(Entity),
}

#[derive(Resource)]
struct PlayerData {
    dock_state: DockState,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            dock_state: DockState::default(),
        }
    }
}

#[derive(Resource)]
struct Config {
    saved: bool,
    values: ConfigValues,
}

const CONFIG_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Clone, Copy, bevy::reflect::TypeUuid, bevy::reflect::TypePath)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
struct ConfigValues {
    drag_c: f32,
    avg_boat_height: f32,
    floating_c: f32,
    drag_ang_c: f32,
    light_dir_color: Color,
    light_amb_color: Color,
    light_dir_lum: f32,
    light_amb_lum: f32,
}

impl Default for ConfigValues {
    fn default() -> Self {
        Self {
            drag_c: 0.05,
            avg_boat_height: 1.,
            floating_c: 1.,
            drag_ang_c: 0.05,
            light_dir_color: Color::rgb(0.98, 0.97, 0.8),
            light_dir_lum: 50_000.,
            light_amb_color: Color::rgb(0.5, 0.5, 0.8),
            light_amb_lum: 1.,
        }
    }
}

fn test_ui_move(
    mut ui_query: Query<&mut Transform, (With<Node>, Without<Sensor>)>,
    island_sensor_query: Query<&Transform, (With<Sensor>, Without<Node>)>,
) {
    let mut ui = ui_query.single_mut();
    let trans = island_sensor_query.iter().next().unwrap();

    *ui = *trans;
}

fn dock_menu_2(
    mut cmd: Commands,
    mut dock_reader: EventReader<Dock>,
    mut dock_menu: Query<Entity, With<DockMenu>>,
    assets: Res<AssetsNonvital>,
) {
    for event in dock_reader.iter() {
        match event {
            Dock::Docking => {
                if dock_menu.is_empty() {
                    cmd.spawn(DockMenu)
                        .insert(NodeBundle {
                            style: Style {
                                width: Val::Percent(50.0),
                                height: Val::Percent(25.0),
                                position_type: PositionType::Absolute,
                                left: Val::Percent(25.),
                                top: Val::Percent(25.),
                                justify_content: JustifyContent::SpaceEvenly,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|cmd| {
                            Card {
                                person_name: "Bob Arne".into(),
                                task: "Do you want to buy 5 bananas?".into(),
                            }
                            .spawn_node(
                                false,
                                cmd,
                                assets.font.clone(),
                            );
                        })
                        .with_children(|cmd| {
                            Card {
                                person_name: "Asbjørn Johann".into(),
                                task: "Do you want to buy 5 bananas?".into(),
                            }
                            .spawn_node(true, cmd, assets.font.clone());
                        })
                        .with_children(|cmd| {
                            Card {
                                person_name: "Bob Arne".into(),
                                task: "Do you want to buy 5 bananas?".into(),
                            }
                            .spawn_node(
                                false,
                                cmd,
                                assets.font.clone(),
                            );
                        });
                }
            }
            Dock::UnDocking => {
                if !dock_menu.is_empty() {
                    cmd.entity(dock_menu.single()).despawn();
                }
            }
        }
    }
}

fn wire_dock_events(
    mut cmd: Commands,
    mut sensor_query: Query<(Entity, &Transform), With<Sensor>>,
    mut dock_menu_query: Query<
        (&mut Visibility, &mut Transform),
        (With<DockMenu>, Without<Sensor>),
    >,
    mut player_query: Query<&Velocity, With<Player>>,
    mut player_data: ResMut<PlayerData>,
    mut dock_writer: EventWriter<Dock>,
) {
    const MAX_DOCK_VEL: f32 = 0.1;
    const MIN_UNDOCK_VEL: f32 = 0.5;

    let mut speed = length_xz(&player_query.single_mut().linvel);
    match player_data.dock_state {
        DockState::TooFar => {}
        DockState::CloseTo(sensor) => {
            if speed < MAX_DOCK_VEL {
                player_data.dock_state = DockState::DockedTo(sensor);
                dock_writer.send(Dock::Docking);
            }
        }
        DockState::DockedTo(sensor) => {
            if MIN_UNDOCK_VEL < speed {
                player_data.dock_state = DockState::CloseTo(sensor);
                dock_writer.send(Dock::UnDocking);
            }
        }
    }
}

fn wire_sensor_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut dock_write: EventWriter<Dock>,
    mut player_query: Query<Entity, With<Player>>,
    mut sensor_query: Query<Entity, With<Sensor>>,
    mut player_data: ResMut<PlayerData>,
) {
    for event in collision_events.iter() {
        let mut player = player_query.single_mut();
        match event {
            CollisionEvent::Started(entity1, entity2, ..) if *entity1 == player => {
                for sensor in sensor_query.iter() {
                    if sensor == *entity2 {
                        player_data.dock_state = DockState::CloseTo(sensor.clone());
                    }
                }
            }
            CollisionEvent::Stopped(entity1, entity2, ..) if *entity1 == player => {
                for sensor in sensor_query.iter() {
                    if sensor == *entity2 {
                        player_data.dock_state = DockState::TooFar;
                    }
                }
            }
            _ => {}
        }
    }
}

fn update_values(
    config: Res<Config>,
    mut events: EventReader<ConfigValuesChanged>,
    mut light_amb: ResMut<AmbientLight>,
    mut light_dir: Query<&mut DirectionalLight>,
) {
    for _ in events.iter() {
        let mut light_dir = light_dir.single_mut();
        light_amb.color = config.values.light_amb_color;
        light_amb.brightness = config.values.light_amb_lum;
        light_dir.color = config.values.light_dir_color;
        light_dir.illuminance = config.values.light_dir_lum;
        light_dir.shadows_enabled = true;
    }
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
        ActiveEvents::COLLISION_EVENTS,
    ));
    cmd.spawn(Camera);
    cmd.insert_resource(AmbientLight {
        color: Color::rgb(0.5, 0.5, 0.8),
        brightness: 1.0,
    });
    cmd.insert_resource(DirectionalLightShadowMap { size: 4090 });
    cmd.insert_resource(Config {
        saved: true,
        values: ConfigValues::default(),
    });
    // cmd.spawn((
    //     DockMenu,
    //     // NodeBundle {
    //     //     style: Style {
    //     //         width: Val::Percent(50.0),
    //     //         height: Val::Percent(50.0),
    //     //         position_type: PositionType::Absolute,
    //     //         left: Val::Percent(25.),
    //     //         top: Val::Percent(25.),
    //     //         justify_content: JustifyContent::SpaceAround,
    //     //         align_items: AlignItems::Center,
    //     //         ..default()
    //     //     },
    //     //     visibility: Visibility::Hidden,
    //     //     background_color: Color::ANTIQUE_WHITE.into(),
    //     //     ..default()
    //     // },
    // ));
    cmd.insert_resource(PlayerData::default());
}

// fn load_config(mut config: ResMut<Config>) {
//     match std::fs::read_to_string("config.json") {
//         Ok(str) => match serde_json::from_str(&str) {
//             Ok(v) => config.values = v,

//             Err(e) => error!("failed to parse json file; {e}"),
//         },
//         Err(e) => error!("failed to read config file: {e}"),
//     }
// }

fn start_loading_assets(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    player: Query<Entity, With<Player>>,
    camera: Query<Entity, With<Camera>>,
    // dock_menu: Query<Entity, With<DockMenu>>,
) {
    // these assets are vital, and the rest of the program need to wait for them

    cmd.insert_resource(AssetPool {
        bboxes: asset_server.load("bboxes.glb"),
        font: asset_server.load("skulls-and-crossbones.ttf"),
        config: asset_server.load(CONFIG_NAME),
    });

    cmd.insert_resource(AssetsVital {
        bboxes: asset_server.load("bboxes.glb"),
    });

    cmd.insert_resource(AssetsNonvital {
        font: asset_server.load("skulls-and-crossbones.ttf"),
    });

    //TODO: this directional light is used before it's guaranteed loaded, but for some reason it doesn't cause a crash
    cmd.spawn(SceneBundle {
        scene: asset_server.load("lights.glb#Scene0"),
        ..default()
    });

    cmd.entity(camera.single()).insert(SceneBundle {
        scene: asset_server.load("cameras.glb#Scene0"),
        ..default()
    });

    // these assets are not vital, so the rest of the program does not need to wait for them
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
    // cmd.entity(dock_menu.single()).insert(SceneBundle {
    //     scene: asset_server.load("persons.glb#Scene0"),
    //     visibility: Visibility::Hidden,
    //     ..default()
    // });
}

fn check_load_state(
    asset_server: Res<AssetServer>,
    asset_pool: Res<AssetPool>,
    mut next_state: ResMut<NextState<AssetState>>,
) {
    use bevy::asset::LoadState::*;

    let config_load_state = asset_server.get_load_state(asset_pool.config.id());
    match config_load_state {
        Failed => warn!("config file failed to load"),
        _ => {}
    }
    let load_states = [
        asset_server.get_load_state(asset_pool.bboxes.id()),
        asset_server.get_load_state(asset_pool.font.id()),
    ];

    if load_states.contains(&Failed) {
        error!("an asset failed to load");
    }

    if load_states
        .iter()
        .chain(once(&config_load_state))
        .all(|v| matches!(v, Loaded) | matches!(v, Failed))
    {
        next_state.set(AssetState::Loaded);
    }
}

fn check_if_vital_assets_loaded(
    asset_server: Res<AssetServer>,
    handles: Res<AssetsVital>,
    mut next_state: ResMut<NextState<AssetState>>,
) {
    match asset_server.get_group_load_state([handles.bboxes.id()]) {
        bevy::asset::LoadState::Loaded => next_state.set(AssetState::Loaded),
        bevy::asset::LoadState::Failed => next_state.set(AssetState::Failed),
        _ => {}
    }
}

fn on_loaded_general(
    mut light: Query<&mut DirectionalLight>,
    mut writer: EventWriter<ConfigValuesChanged>,
    mut config: ResMut<Config>,
    config_asset: Res<Assets<ConfigValues>>,
    asset_pool: Res<AssetPool>,
) {
    match config_asset.get(&asset_pool.config) {
        None => {
            warn!("config not loaded");
        }
        Some(v) => {
            config.values = *v;
        }
    }
    // make use the config values are used once loaded
    writer.send(ConfigValuesChanged);

    let mut light = light.single_mut();
    light.shadows_enabled = true;
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
    // convert colliders
    let gltf = assets_gltf.get(&handles.bboxes).unwrap();

    let mut colliders_trimesh = gltf
        .named_nodes
        .iter()
        .filter_map(|(k, v)| match k.strip_suffix("-trimesh") {
            None => None,
            Some(stripped) => {
                let node = assets_gltf_nodes.get(&v).unwrap();

                let transform = TransformBundle::from(node.transform);
                let mesh = assets_mesh
                    .get(
                        &assets_gltf_mesh
                            .get(&node.mesh.as_ref().unwrap())
                            .unwrap()
                            .primitives[0]
                            .mesh,
                    )
                    .unwrap();
                let collider =
                    Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();
                Some((stripped.into(), (transform, collider)))
            }
        })
        .collect::<HashMap<String, (TransformBundle, Collider)>>();

    let mut colliders_cylinder = gltf
        .named_nodes
        .iter()
        .filter_map(|(k, v)| match k.strip_suffix("-cylinder") {
            None => None,
            Some(stripped) => {
                let node = assets_gltf_nodes.get(&v).unwrap();

                let transform = TransformBundle::from(node.transform);

                let collider = Collider::cylinder(3., 1.);
                Some((stripped.into(), (transform, collider)))
            }
        })
        .collect::<HashMap<String, (TransformBundle, Collider)>>();

    // spawn colliders
    cmd.entity(player.single())
        .insert(colliders_trimesh["boat"].1.clone());

    let island1 = colliders_trimesh["island-1"].clone();
    let island2 = colliders_trimesh["island-2"].clone();
    cmd.spawn((RigidBody::Fixed, island1.0, island1.1));
    cmd.spawn((RigidBody::Fixed, island2.0, island2.1));

    let island1 = colliders_cylinder["island-1"].clone();
    let island2 = colliders_cylinder["island-2"].clone();
    cmd.spawn((Sensor, island1.0, island1.1)); //, ActiveEvents::COLLISION_EVENTS));
    cmd.spawn((Sensor, island2.0, island2.1));

    cmd.remove_resource::<AssetsVital>();
}

fn update_ui(
    state: Res<State<AssetState>>,
    mut contexts: EguiContexts,
    mut writer_config_save: EventWriter<ConfigSave>,
    mut writer_config_changed: EventWriter<ConfigValuesChanged>,
    mut config: ResMut<Config>,
    mut debug_mode: ResMut<DebugRenderContext>,
    player_data: Res<PlayerData>,
    player_query: Query<&Velocity, With<Player>>,
) {
    use egui::*;
    //TODO: add a reload config button
    egui::Window::new("debug control panel").show(contexts.ctx_mut(), |ui| {
        let player_speed_xz = length_xz(&player_query.single().linvel);

        match state.get() {
            AssetState::Loading => ui.label("loading assets for vital functions"),
            AssetState::Loaded => ui.label("press arrow keys to move the boat"),
            AssetState::Failed => {
                ui.label("assets failed to load for some reason, check console for detailed errors")
            }
        };
        ui.separator();
        // the reason these are not combined with || operator is that the compiler optimizes, and then only the first will show

        let mut changed = false;

        ui.collapsing("physics", |ui| {
            if ui
                .add(egui::Slider::new(&mut config.values.drag_c, 0.0..=0.3).text("drag c"))
                .changed()
                | ui.add(
                    egui::Slider::new(&mut config.values.floating_c, 0.0..=5.).text("floating c"),
                )
                .changed()
                | ui.add(
                    egui::Slider::new(&mut config.values.drag_ang_c, 0.0..=0.3)
                        .text("angular drag c"),
                )
                .changed()
                | ui.add(
                    egui::Slider::new(&mut config.values.avg_boat_height, 0.0..=5.)
                        .text("average boat height"),
                )
                .changed()
            {
                changed = true;
            }

            ui.checkbox(&mut debug_mode.enabled, "render bbox");

            ui.label(format!("docking state: {:?}", player_data.dock_state));
            ui.label(format!("player speed: {:.2}", player_speed_xz,));
        }); // diractional light
        ui.collapsing("graphics", |ui| {
            if ui
                .add(
                    egui::Slider::new(&mut config.values.light_dir_lum, 0.0..=100_000.)
                        .text("directional light illuminance"),
                )
                .changed()
                | ui.add(
                    egui::Slider::new(&mut config.values.light_amb_lum, 0.0..=3.)
                        .text("ambient light illuminance"),
                )
                .changed()
            {
                changed = true;
            }

            // directional light
            ui.horizontal(|ui| {
                let mut buf = [
                    config.values.light_dir_color.r(),
                    config.values.light_dir_color.g(),
                    config.values.light_dir_color.b(),
                ];
                if ui.color_edit_button_rgb(&mut buf).changed() {
                    changed = true;
                    config.values.light_dir_color = buf.into();
                }
                ui.label("directional light color")
            });

            // ambient light
            ui.horizontal(|ui| {
                let mut buf = [
                    config.values.light_amb_color.r(),
                    config.values.light_amb_color.g(),
                    config.values.light_amb_color.b(),
                ];
                if ui.color_edit_button_rgb(&mut buf).changed() {
                    changed = true;
                    config.values.light_amb_color = buf.into();
                }
                ui.label("ambient light color")
            });
        });

        // make sure config values is updated, and the file is saved
        config.saved &= !changed;
        if changed {
            writer_config_changed.send(ConfigValuesChanged);
        }

        if ui
            .add_enabled(
                !config.saved,
                if config.saved {
                    egui::Button::new("config saved")
                } else {
                    egui::Button::new("save config")
                },
            )
            .clicked()
        {
            writer_config_save.send(ConfigSave);
        }
    });
}

fn save_config(mut config: ResMut<Config>, mut events: EventReader<ConfigSave>) {
    //TODO: check if writing json works on web
    for _ in events.iter() {
        match std::fs::write(
            "assets/".to_owned() + CONFIG_NAME,
            json!(config.values).to_string(),
        ) {
            Ok(_) => config.saved = true,
            Err(e) => error!("could not save config file: {e}"),
        }
    }
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
    config: Res<Config>,
) {
    //TODO: Transform trenger ikke være mut her
    for (trans, mut vel) in floating_objects.iter_mut() {
        // # bouancy from water
        //TODO: do this in a continuous way instead, without if statements; just for fun and practice ofcousrse
        let y = trans.translation.y;
        let mut v = 0.;
        if y < 0. {
            if -y > config.values.avg_boat_height {
                v = config.values.avg_boat_height * config.values.floating_c;
            } else {
                v = -y * config.values.floating_c;
            }
        }
        vel.linvel.y += v;
        let inverse = -vel.linvel;
        vel.linvel += inverse * config.values.drag_c;

        // # drag from turning and moving forward
        let speed = vel.linvel.length();
        if 0.001 < speed {
            let normal = vel.linvel.normalize();
            vel.linvel -= normal * config.values.drag_c * speed;
        }

        let speed = vel.angvel.length();
        if 0.001 < speed {
            let normal = vel.angvel.normalize();
            vel.angvel -= normal * config.values.drag_ang_c * speed
        }
    }
}
