use avian3d::{PhysicsPlugins, prelude::*};
use bevy::{
    asset::AssetMetaCheck,
    dev_tools::fps_overlay::*,
    ecs::system::command,
    gltf::GltfMesh,
    input::{common_conditions::input_just_pressed, keyboard::KeyboardInput},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_mod_outline::{
    GenerateOutlineNormalsSettings, OutlineMeshExt, OutlineMode, OutlinePlugin, OutlineVolume,
};
use rand::random_range;

// wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "shmoop_manager"  ./target/wasm32-unknown-unknown/debug/save_them_fools.wasm
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin::default(),
            OutlinePlugin,
            // FpsOverlayPlugin {
            //     config: FpsOverlayConfig {
            //         enabled: true,
            //         ..default()
            //     },
            // },
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameState::Loading)
        .insert_resource(MapBounds {
            half_size: Vec3::new(5.0, 0.0, 5.0),
        })
        // .insert_resource(AmbientLight::NONE)
        .add_systems(
            Startup,
            (setup_system, load_gltf, loading_screen_system).chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                shmoop_moving_to_destination_system.run_if(resource_equals(GameState::Playing)),
                shmoop_destination_selection_system.run_if(resource_equals(GameState::Playing)),
                shmoop_dragging_system.run_if(resource_equals(GameState::Playing)),
                hunger_system.run_if(resource_equals(GameState::Playing)),
                map_shrinking_system.run_if(resource_equals(GameState::Playing)),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                destination_time_system.run_if(resource_equals(GameState::Playing)),
                destination_abandoning_system.run_if(resource_equals(GameState::Playing)),
                select_system.run_if(resource_equals(GameState::Playing)),
                shmoop_fall_death_system.run_if(resource_equals(GameState::Playing)),
                pickup_interaction_system.run_if(resource_equals(GameState::Playing)),
                food_store_interaction_system.run_if(resource_equals(GameState::Playing)),
                spawn_gltf_scene.run_if(resource_equals(GameState::Loading)),
                reset_game_system.run_if(resource_equals(GameState::PendingStart)),
                despawn_system.run_if(resource_equals(GameState::Playing)),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                shmoop_count_system.run_if(resource_equals(GameState::Playing)),
                start_screen_system.run_if(resource_equals(GameState::StartScreen)),
                loading_screen_system.run_if(resource_equals(GameState::PendingStart)),
                loading_screen_system.run_if(resource_equals(GameState::Spawning)),
                restart_system
                    .run_if(|game_state: Res<GameState>| {
                        *game_state != GameState::Loading && *game_state != GameState::PendingStart
                    })
                    .run_if(input_just_pressed(KeyCode::Space)),
            )
                .chain(),
        )
        .run();
}

#[derive(Resource, PartialEq)]
enum GameState {
    Loading,
    StartScreen,
    Playing,
    PendingStart,
    Spawning,
}

#[derive(Resource)]
struct ShmoopGltf(Handle<Gltf>);

#[derive(Resource)]
struct FoodGltf(Handle<Gltf>);

#[derive(Resource)]
struct PlatformGltf(Handle<Gltf>);

#[derive(Resource)]
struct ShipGltf(Handle<Gltf>);

fn load_gltf(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ShmoopGltf(asset_server.load("Capybara.glb")));
    commands.insert_resource(FoodGltf(asset_server.load("Watermelon.glb")));
    commands.insert_resource(PlatformGltf(asset_server.load("Platform.glb")));
    commands.insert_resource(ShipGltf(asset_server.load("Ship.glb")));
}

fn spawn_gltf_scene(
    shmoop_gltf: Res<ShmoopGltf>,
    food_gltf: Res<FoodGltf>,
    platform_gltf: Res<PlatformGltf>,
    ship_gltf: Res<ShipGltf>,
    mut game_state: ResMut<GameState>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut commands: Commands,
) {
    if gltf_assets.get(&shmoop_gltf.0).is_some()
        && gltf_assets.get(&food_gltf.0).is_some()
        && gltf_assets.get(&platform_gltf.0).is_some()
        && gltf_assets.get(&ship_gltf.0).is_some()
    {
        *game_state = GameState::StartScreen;

        // Ship
        {
            let gltf = gltf_assets.get(&ship_gltf.0).unwrap();
            let ship_mesh_handle = gltf.named_meshes.get("Ship").unwrap();
            let ship_gltf_mesh = gltf_meshes.get(ship_mesh_handle).unwrap();
            let ship_primitive = ship_gltf_mesh.primitives.get(0).unwrap();
            let parus_primitive = ship_gltf_mesh.primitives.get(1).unwrap();

            let door_mesh_handle = gltf.named_meshes.get("Door").unwrap();
            let door_gltf_mesh = gltf_meshes.get(door_mesh_handle).unwrap();
            let door_primitive = door_gltf_mesh.primitives.get(0).unwrap();

            commands.spawn((
                ShipFloor,
                CanBeDraggedOn,
                RigidBody::Static,
                ColliderConstructor::TrimeshFromMesh,
                Mesh3d(ship_primitive.mesh.clone()),
                MeshMaterial3d(ship_primitive.material.clone().unwrap()),
                Transform::from_translation(Vec3::new(-6.8, 0.5, 0.0))
                    .with_scale(Vec3::splat(0.5))
                    .with_rotation(Quat::from_rotation_y(1.0 * std::f32::consts::PI)),
                children![(
                    RigidBody::Static,
                    ColliderConstructor::TrimeshFromMesh,
                    Mesh3d(parus_primitive.mesh.clone()),
                    MeshMaterial3d(parus_primitive.material.clone().unwrap()),
                    // Transform::from_translation(Vec3::new(-6.5, 1.0, 0.0))
                    //     .with_scale(Vec3::splat(0.5))
                    //     .with_rotation(Quat::from_rotation_y(1.0 * std::f32::consts::PI)),
                )],
            ));

            commands.spawn((
                ShipFloor,
                CanBeDraggedOn,
                RigidBody::Static,
                ColliderConstructor::TrimeshFromMesh,
                Mesh3d(door_primitive.mesh.clone()),
                MeshMaterial3d(door_primitive.material.clone().unwrap()),
                Transform::from_translation(Vec3::new(-6.0, 0.00, 0.0))
                    .with_scale(Vec3::splat(0.5))
                    .with_rotation(Quat::from_rotation_z(-std::f32::consts::PI / 1.95)),
                // Transform::from_translation(Vec3::new(-6.5, 1.0, 0.0))
                //     .with_scale(Vec3::splat(0.5))
                //     .with_rotation(Quat::from_rotation_y(1.0 * std::f32::consts::PI)),
            ));
        }
    }
}

fn setup_system(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // 6 world units per pixel of window height.
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 5.0, -5.0).looking_at(Vec3::new(0.0, 2.5, 0.0), Vec3::Y),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::IDENTITY.looking_at(Vec3::new(-0.6, -1.0, 0.4).normalize(), Vec3::Y),
    ));
}

fn restart_system(mut game_state: ResMut<GameState>) {
    *game_state = GameState::PendingStart;
}

fn loading_screen_system(mut commands: Commands, text_query: Query<Entity, With<MyText>>) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        MyText,
        Text::new("Loading"),
        TextColor(Color::srgb(0.0, 0.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(49.0),
            ..default()
        },
    ));
}

fn start_screen_system(mut commands: Commands, text_query: Query<Entity, With<MyText>>) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        MyText,
        Text::new("Press SPACE to start!"),
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(40.0),
            ..default()
        },
    ));

    commands.spawn((
        MyText,
        Text::new(concat!(
            "Watermelon by Kenney (https://poly.pizza/m/lJIfjMl47l)\n\n",
            "Capybara by Poly by Google [CC-BY] (https://creativecommons.org/licenses/by/3.0/)\nvia Poly Pizza (https://poly.pizza/m/66d-mKAgF17)\n",
        )),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));
    commands.spawn((
        MyText,
        Text::new("Made with Bevy Engine"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.),
            top: Val::Px(12.),
            ..default()
        },
    ));
}
fn reset_game_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    restartables_query: Query<Entity, With<Restartable>>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    shmoop_gltf: Res<ShmoopGltf>,
    food_gltf: Res<FoodGltf>,
    platform_gltf: Res<PlatformGltf>,
) {
    for entity in restartables_query.iter() {
        commands.entity(entity).despawn();
    }

    let map_bounds = MapBounds {
        half_size: Vec3::new(5.0, 0.0, 5.0),
    };
    commands.insert_resource(map_bounds.clone());

    // plane
    {
        let gltf = gltf_assets.get(&platform_gltf.0).unwrap();
        let (_, mesh_handle) = gltf.named_meshes.iter().next().unwrap();
        let gltf_mesh = gltf_meshes.get(mesh_handle).unwrap();
        let primitive = gltf_mesh.primitives.first().unwrap();
        let width = 1.75;
        let even_offset = width / 2.0;
        let height_offset = 1.5;

        let spawn_positions = [
            Vec3::new(0.0, -0.1, 0.0),
            Vec3::new(width, -0.1, 0.0),
            Vec3::new(width * 2.0, -0.1, 0.0),
            Vec3::new(width * 3.0, -0.1, 0.0),
            Vec3::new(-width, -0.1, 0.0),
            Vec3::new(-width * 2.0, -0.1, 0.0),
            Vec3::new(-width * 3.0, -0.1, 0.0),
            Vec3::new(even_offset + 0.0, -0.1, height_offset),
            Vec3::new(even_offset + width, -0.1, height_offset),
            Vec3::new(even_offset + width * 2.0, -0.1, height_offset),
            // Vec3::new(even_offset + width * 3.0, -0.1, height_offset),
            Vec3::new(even_offset + -width, -0.1, height_offset),
            Vec3::new(even_offset + -width * 2.0, -0.1, height_offset),
            Vec3::new(even_offset + -width * 3.0, -0.1, height_offset),
            Vec3::new(0.0, -0.1, height_offset * 2.0),
            Vec3::new(width, -0.1, height_offset * 2.0),
            Vec3::new(width * 2.0, -0.1, height_offset * 2.0),
            Vec3::new(width * 3.0, -0.1, height_offset * 2.0),
            Vec3::new(-width, -0.1, height_offset * 2.0),
            Vec3::new(-width * 2.0, -0.1, height_offset * 2.0),
            Vec3::new(-width * 3.0, -0.1, height_offset * 2.0),
            Vec3::new(even_offset + 0.0, -0.1, height_offset * 3.0),
            Vec3::new(even_offset + width, -0.1, height_offset * 3.0),
            Vec3::new(even_offset + width * 2.0, -0.1, height_offset * 3.0),
            Vec3::new(even_offset + width * 3.0, -0.1, height_offset * 3.0),
            Vec3::new(even_offset + -width, -0.1, height_offset * 3.0),
            Vec3::new(even_offset + -width * 2.0, -0.1, height_offset * 3.0),
            Vec3::new(even_offset + -width * 3.0, -0.1, height_offset * 3.0),
            Vec3::new(0.0, -0.1, height_offset * 4.0),
            Vec3::new(width, -0.1, height_offset * 4.0),
            // Vec3::new(width * 2.0, -0.1, height_offset * 4.0),
            // Vec3::new(width * 3.0, -0.1, height_offset * 4.0),
            Vec3::new(-width, -0.1, height_offset * 4.0),
            // Vec3::new(-width * 2.0, -0.1, height_offset * 4.0),
            // Vec3::new(-width * 3.0, -0.1, height_offset * 4.0),
            //
            Vec3::new(even_offset + 0.0, -0.1, -height_offset),
            Vec3::new(even_offset + width, -0.1, -height_offset),
            Vec3::new(even_offset + width * 2.0, -0.1, -height_offset),
            Vec3::new(even_offset + width * 3.0, -0.1, -height_offset),
            Vec3::new(even_offset + -width, -0.1, -height_offset),
            Vec3::new(even_offset + -width * 2.0, -0.1, -height_offset),
            Vec3::new(even_offset + -width * 3.0, -0.1, -height_offset),
            Vec3::new(0.0, -0.1, -height_offset * 2.0),
            Vec3::new(width, -0.1, -height_offset * 2.0),
            Vec3::new(width * 2.0, -0.1, -height_offset * 2.0),
            Vec3::new(width * 3.0, -0.1, -height_offset * 2.0),
            Vec3::new(-width, -0.1, -height_offset * 2.0),
            Vec3::new(-width * 2.0, -0.1, -height_offset * 2.0),
            Vec3::new(-width * 3.0, -0.1, -height_offset * 2.0),
            Vec3::new(even_offset + 0.0, -0.1, -height_offset * 3.0),
            Vec3::new(even_offset + width, -0.1, -height_offset * 3.0),
            Vec3::new(even_offset + width * 2.0, -0.1, -height_offset * 3.0),
            // Vec3::new(even_offset + width * 3.0, -0.1, -height_offset * 3.0),
            Vec3::new(even_offset + -width, -0.1, -height_offset * 3.0),
            Vec3::new(even_offset + -width * 2.0, -0.1, -height_offset * 3.0),
            Vec3::new(even_offset + -width * 3.0, -0.1, -height_offset * 3.0),
            Vec3::new(0.0, -0.1, -height_offset * 4.0),
            Vec3::new(width, -0.1, -height_offset * 4.0),
            Vec3::new(width * 2.0, -0.1, -height_offset * 4.0),
            // Vec3::new(width * 3.0, -0.1, -height_offset * 4.0),
            Vec3::new(-width, -0.1, -height_offset * 4.0),
            // Vec3::new(-width * 2.0, -0.1, -height_offset * 4.0),
            // Vec3::new(-width * 3.0, -0.1, -height_offset * 4.0),
        ];

        for spawn_position in spawn_positions {
            commands.spawn((
                Ground,
                CanBeDraggedOn,
                Restartable,
                RigidBody::Static,
                ColliderConstructor::ConvexHullFromMesh,
                Mesh3d(primitive.mesh.clone()),
                MeshMaterial3d(primitive.material.clone().unwrap()),
                Transform::from_translation(spawn_position),
                Mass(300.0),
            ));
        }
    }

    // spawn shmoops
    {
        let gltf = gltf_assets.get(&shmoop_gltf.0).unwrap();
        let (_, mesh_handle) = gltf.named_meshes.iter().next().unwrap();
        let gltf_mesh = gltf_meshes.get(mesh_handle).unwrap();
        let primitive = gltf_mesh.primitives.first().unwrap();

        let spawn_positions = [
            Vec3::new(-7.5, 0.5, -1.0),
            Vec3::new(-7.5, 0.5, 0.0),
            Vec3::new(-7.5, 0.5, 1.0),
            Vec3::new(-8.0, 0.5, -1.0),
            Vec3::new(-8.0, 0.5, 0.0),
            Vec3::new(-8.0, 0.5, 1.0),
            Vec3::new(-8.5, 0.5, -1.0),
            Vec3::new(-8.5, 0.5, 0.0),
            Vec3::new(-8.5, 0.5, 1.0),
        ];

        for spawn_position in spawn_positions {
            commands.spawn((
                Shmoop,
                Restartable,
                Hunger { percentage: 100.0 },
                RigidBody::Dynamic,
                OutlineVolume {
                    visible: false,
                    colour: Color::WHITE,
                    width: 1.0,
                },
                OutlineMode::ExtrudeReal,
                ColliderConstructor::RoundCuboid {
                    x_length: 0.8,
                    y_length: 2.5,
                    z_length: 5.1,
                    border_radius: 0.1,
                },
                LockedAxes::new().lock_rotation_x().lock_rotation_z(),
                Mesh3d(primitive.mesh.clone()),
                MeshMaterial3d(primitive.material.clone().unwrap()),
                Transform::from_translation(spawn_position).with_scale(Vec3::splat(0.1)),
                // ExternalAngularImpulse::new(Vec3::new(0.0, 10.0, 0.0)),
            ));
        }
    }

    // Food
    {
        let gltf: &Gltf = gltf_assets.get(&food_gltf.0).unwrap();
        let scene_handle = gltf.scenes.iter().next().unwrap();

        const RADIUS: f32 = 0.25;
        const SPAWN_HEIGHT: f32 = 0.4;
        let spawn_positions = [
            Vec3::new(4.7, SPAWN_HEIGHT, 0.5),
            Vec3::new(3.0, SPAWN_HEIGHT, -4.5),
            Vec3::new(-3.4, SPAWN_HEIGHT, 3.5),
            Vec3::new(-3.1, SPAWN_HEIGHT, 1.8),
            Vec3::new(-3.5, SPAWN_HEIGHT, -1.2),
            Vec3::new(0.7, SPAWN_HEIGHT, -2.6),
            Vec3::new(1.7, SPAWN_HEIGHT, -0.5),
            Vec3::new(1.0, SPAWN_HEIGHT, 2.5),
            Vec3::new(2.7, SPAWN_HEIGHT, 1.4),
        ];

        for spawn_position in spawn_positions {
            commands.spawn((
                FoodStore,
                Restartable,
                CanBeCarried,
                RigidBody::Dynamic,
                Interactable,
                OutlineVolume {
                    visible: false,
                    colour: Color::WHITE,
                    width: 1.0,
                },
                OutlineMode::ExtrudeReal,
                SceneRoot(scene_handle.clone()),
                ColliderConstructor::Sphere { radius: RADIUS },
                Transform::from_translation(spawn_position),
            ));
        }
    }

    // Trees
    {
        let mesh = meshes.add(Cylinder::new(0.1, 2.0).mesh().build());

        let height = 1.2;
        let spawn_positions = [
            Vec3::new(4.5, height, -0.5),
            Vec3::new(2.5, height, -4.5),
            Vec3::new(-3.4, height, -3.5),
            Vec3::new(-1.5, height, 4.9),
            Vec3::new(-2.1, height, -1.8),
            Vec3::new(-2.5, height, 1.2),
            Vec3::new(1.7, height, -2.6),
            Vec3::new(-1.5, height, 0.5),
            Vec3::new(2.0, height, 2.5),
            Vec3::new(-1.2, height, -1.4),
            Vec3::new(5.2, height, 3.4),
            Vec3::new(4.2, height, 1.4),
            Vec3::new(4.4, height, -3.4),
            Vec3::new(4.1, height, 5.4),
            Vec3::new(2.3, height, 3.5),
            Vec3::new(1.3, height, 4.5),
        ];

        for spawn_position in spawn_positions {
            commands.spawn((
                Tree,
                Restartable,
                CanBeCarried,
                RigidBody::Dynamic,
                Interactable,
                OutlineVolume {
                    visible: false,
                    colour: Color::WHITE,
                    width: 1.0,
                },
                OutlineMode::ExtrudeReal,
                ColliderConstructor::Cylinder {
                    radius: 0.1,
                    height: 2.0,
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(materials.add(Color::srgb_u8(139, 69, 19))),
                Transform::from_translation(spawn_position),
            ));
        }
    }

    for (_, mesh) in meshes.iter_mut() {
        mesh.generate_outline_normals(&GenerateOutlineNormalsSettings::default())
            .unwrap();
    }

    commands.insert_resource(GameState::Playing);
}

const PICK_MOUSE_BUTTON: MouseButton = MouseButton::Left;

const HOVER_COLOR: Color = Color::srgba(0.0, 1.0, 0.5, 0.2);
const PICKING_COLOR: Color = Color::WHITE;
const TARGET_SELECTION_COLOR: Color = Color::srgba(0.0, 0.2, 1.0, 0.5);

#[derive(Component, Clone, Copy)]
pub struct Restartable;

#[derive(Component, Clone, Copy)]
pub struct Shmoop;

#[derive(Component, Clone, Copy)]
pub struct Picked;

#[derive(Component, Clone, Copy)]
pub struct Dead;

#[derive(Component, Clone, Copy)]
pub struct Ground;

#[derive(Component, Clone, Copy)]
pub struct CanBeDraggedOn;
#[derive(Component, Clone, Copy)]
pub struct Tree;

#[derive(Component, Clone, Copy)]
pub struct FoodStore;

#[derive(Component, Clone)]
pub struct Storage {
    pub log_positions: Vec<Vec3>,
}

#[derive(Component, Clone, Copy)]
pub struct CanBeCarried;

#[derive(Component, Clone, Copy)]
pub struct Carrying {
    pub entity: Entity,
    pub joint_entity: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub percentage: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Hunger {
    pub percentage: f32,
}

#[derive(Component, Clone, Copy)]
pub struct DestinationTime {
    pub time: f32,
}

#[derive(Component, Clone, Copy)]
pub struct MyText;
#[derive(Component, Clone, Copy)]
pub struct ShipFloor;

#[derive(Component, Clone, Copy)]
pub struct Destructable;

#[derive(Component, Clone, Copy)]
pub struct Interactable;

#[derive(Component, Clone, Copy)]
pub struct ShmoopDestination {
    pub target: Vec3,
}

#[derive(Component, Clone, Copy)]
pub struct ShmoopInteractionTarget {
    pub entity: Entity,
}

#[derive(Resource, Clone, Copy)]
pub struct MapBounds {
    pub half_size: Vec3,
}

fn map_shrinking_system(
    ground: Query<(Entity, &RigidBody, &Position), With<Ground>>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut commands: Commands,
) {
    *timer += time.delta_secs();

    if *timer <= 5.0 {
        return;
    }
    *timer = 0.0;

    let ship_position = Vec3::new(-6.8, 0.5, 0.0);

    let mut most_length: Option<f32> = None;
    let mut most_entity: Option<Entity> = None;
    for (entity, body, position) in ground.iter() {
        if *body != RigidBody::Static {
            continue;
        }
        let length = (position.0 - ship_position).length();
        let Some(most_length2) = most_length else {
            most_length = Some(length);
            most_entity = Some(entity);
            continue;
        };

        if length > most_length2 {
            most_length = Some(length);
            most_entity = Some(entity);
        }
    }

    let Some(most_entity) = most_entity else {
        return;
    };

    commands.entity(most_entity).insert(RigidBody::Dynamic);
}

fn shmoop_count_system(
    mut commands: Commands,
    shmoops_query: Query<&Position, (With<Shmoop>, Without<Tree>, Without<Dead>)>,
    trees_query: Query<&Position, (With<Tree>, Without<Shmoop>, Without<Dead>)>,
    text_query: Query<Entity, With<MyText>>,
    keyboard_keys: Res<ButtonInput<KeyCode>>,
) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }

    let shmoops_count = shmoops_query.iter().count();
    if shmoops_count == 0 {
        commands.spawn((
            MyText,
            Text::new("You've lost all the shmips. Oops!\n"),
            TextColor(Color::srgb(1.0, 0.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                left: Val::Percent(35.0),
                ..default()
            },
        ));
        return;
    }

    let mut collected_trees_count: u32 = 0;

    let mut all_trees_in = true;
    for position in trees_query.iter() {
        if is_object_on_ship(position) {
            collected_trees_count += 1;
        } else {
            all_trees_in = false;
        }
    }

    let mut all_shmoops_in = true;
    for position in shmoops_query.iter() {
        if !is_object_on_ship(position) {
            all_shmoops_in = false;
            break;
        }
    }

    if all_shmoops_in && all_trees_in {
        commands.spawn((MyText,
            Text::new(format!("All {shmoops_count} shmips are on the ship!\n Logs collected {collected_trees_count}")),
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                left: Val::Percent(35.0),
                ..default()
            },
        ));

        return;
    }

    commands.spawn((
        MyText,
        Text::new(format!(
            "You have {shmoops_count} shmips left.\nLogs collected: {collected_trees_count}"
        )),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));

    if keyboard_keys.pressed(KeyCode::Escape) {
        commands.spawn((
            MyText,
            Text::new(concat!(
                "Hold left mouse button to select a shmip.\n",
                "Release the button where you want the shmip to go.\n",
                "Release the mouse button on a log to pick it up.\n",
                "Collect all the logs and shmips on the ship to finish.\n",
                "Press SPACE to restart.\n",
            )),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.),
                left: Val::Px(12.),
                ..default()
            },
        ));
    } else {
        commands.spawn((
            MyText,
            Text::new("Hold ESCAPE to see the instruction"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.),
                left: Val::Px(12.),
                ..default()
            },
        ));
    }
}

fn is_object_on_ship(position: &Position) -> bool {
    position.0.x <= -6.0 && position.0.y >= 0.0
}
fn shmoop_moving_to_destination_system(
    time: Res<Time>,
    mut commands: Commands,
    mut shmoop_query: Query<
        (
            Entity,
            &ShmoopDestination,
            &Position,
            &Rotation,
            &mut LinearVelocity,
            &mut AngularVelocity,
            Option<&ShmoopInteractionTarget>,
            Option<&Carrying>,
        ),
        (With<Shmoop>, Without<Picked>),
    >,
) {
    const MOVING_SPEED: f32 = 50.0;
    for (
        shmoop_entity,
        destination,
        position,
        rotation,
        mut linear_velocity,
        mut angular_velocity,
        interaction_target,
        carrying,
    ) in shmoop_query.iter_mut()
    {
        let direction = destination.target - position.0;
        if direction.length() > 0.5 || interaction_target.is_some() {
            let direction = direction.normalize_or_zero() * time.delta_secs();
            linear_velocity.0.x = direction.x * MOVING_SPEED;
            linear_velocity.0.z = direction.z * MOVING_SPEED;

            let current_forward = rotation.0.mul_vec3(Vec3::Z).normalize_or_zero();
            let target_forward = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();

            if target_forward.length_squared() > 0.0 && current_forward.length_squared() > 0.0 {
                let rotation_axis = current_forward.cross(target_forward);
                let angle = current_forward.angle_between(target_forward);
                if angle > 0.01 {
                    let angular_speed = 1.0; // Adjust rotation speed as needed
                    angular_velocity.0 = rotation_axis.normalize_or_zero() * angle * angular_speed;
                } else {
                    angular_velocity.0 = Vec3::ZERO;
                }
            }
            continue;
        }
        linear_velocity.0 = Vec3::ZERO;
        commands.entity(shmoop_entity).remove::<ShmoopDestination>();
        commands.entity(shmoop_entity).remove::<DestinationTime>();
        println!("Shmoop {} arrived at destination", shmoop_entity);

        if let Some(carrying) = carrying {
            commands.entity(carrying.joint_entity).despawn();

            commands.entity(shmoop_entity).remove::<Carrying>();
        }
    }
}

fn pickup_interaction_system(
    mut commands: Commands,
    shmoops_query: Query<(Entity, &ShmoopInteractionTarget), (With<Shmoop>, Without<Picked>)>,
    interactables_query: Query<Entity, (Without<Shmoop>, With<Interactable>, With<CanBeCarried>)>,
    collisions: Collisions,
) {
    for (shmoop_entity, interaction_target) in shmoops_query.iter() {
        if !interactables_query.contains(interaction_target.entity) {
            continue;
        };

        let Some(collision) = collisions.get(shmoop_entity, interaction_target.entity) else {
            continue;
        };
        let Some(manifold) = collision.manifolds.get(0) else {
            continue;
        };
        let Some(contact_point) = manifold.points.get(0) else {
            continue;
        };

        let shmoop_point = if collision.collider1 == shmoop_entity {
            contact_point.local_point1
        } else {
            contact_point.local_point2
        };

        let interactable_point = if collision.collider1 == interaction_target.entity {
            contact_point.local_point1
        } else {
            contact_point.local_point2
        };

        let joint_entity = commands
            .spawn(
                DistanceJoint::new(shmoop_entity, interaction_target.entity)
                    .with_compliance(0.5)
                    .with_local_anchor_1(shmoop_point)
                    .with_local_anchor_2(interactable_point),
            )
            .id();

        commands.entity(shmoop_entity).insert(Carrying {
            entity: interaction_target.entity,
            joint_entity,
        });

        commands
            .entity(shmoop_entity)
            .remove::<ShmoopInteractionTarget>();
        commands.entity(shmoop_entity).remove::<ShmoopDestination>();
        commands.entity(shmoop_entity).remove::<DestinationTime>();

        println!(
            "Shmoop {} picked up interactable {}",
            shmoop_entity, interaction_target.entity
        );
    }
}

fn food_store_interaction_system(
    mut commands: Commands,
    mut shmoops_query: Query<
        (Entity, &ShmoopInteractionTarget, &mut Hunger),
        (With<Shmoop>, Without<Picked>),
    >,
    food_store_query: Query<Entity, (Without<Shmoop>, With<Interactable>, With<FoodStore>)>,
    collisions: Collisions,
) {
    for (shmoop_entity, interaction_target, mut hunger) in shmoops_query.iter_mut() {
        let Ok(food_store_entity) = food_store_query.get(interaction_target.entity) else {
            continue;
        };

        if collisions.get(shmoop_entity, food_store_entity).is_none() {
            continue;
        }

        hunger.percentage = 100.0;

        commands
            .entity(shmoop_entity)
            .remove::<ShmoopInteractionTarget>();
        commands.entity(shmoop_entity).remove::<ShmoopDestination>();
        commands.entity(shmoop_entity).remove::<DestinationTime>();

        println!(
            "Shmoop {} stored interactable {}",
            shmoop_entity, interaction_target.entity
        );
    }
}

fn hunger_system(
    time: Res<Time>,
    mut commands: Commands,
    mut shmoops_query: Query<(Entity, &mut Hunger, Option<&Carrying>), With<Shmoop>>,
    food_store_query: Query<
        (Entity, &Position),
        (Without<Shmoop>, With<Interactable>, With<FoodStore>),
    >,
) {
    for (shmoop_entity, mut hunger, carrying) in shmoops_query.iter_mut() {
        let mut amount = 4.0 * time.delta_secs();

        if carrying.is_some() {
            amount *= 7.0;
        }

        hunger.percentage -= amount;

        if hunger.percentage <= 0.0 {
            let Ok((food_store_entity, food_store_position)) = food_store_query.single() else {
                continue;
            };

            commands
                .entity(shmoop_entity)
                .insert(ShmoopInteractionTarget {
                    entity: food_store_entity,
                });
            commands.entity(shmoop_entity).insert((
                ShmoopDestination {
                    target: food_store_position.0,
                },
                DestinationTime { time: 0.0 },
            ));
        }
    }
}

fn shmoop_fall_death_system(
    mut commands: Commands,
    map_bounds: Res<MapBounds>,
    mut shmoops_query: Query<(Entity, &Position, Option<&Dead>), With<Shmoop>>,
) {
    for (entity, position, dead) in shmoops_query.iter_mut() {
        if position.0.y < map_bounds.half_size.y {
            if dead.is_none() {
                commands.entity(entity).insert(Dead);
                println!("Shmoop {} is dead", entity);
            }
        } else if dead.is_some() {
            commands.entity(entity).remove::<Dead>();
            println!("Shmoop {} is saved", entity);
        }
    }
}

fn despawn_system(mut commands: Commands, query: Query<(Entity, &Position)>) {
    const DESPAWN_DEPTH: f32 = -50.0;

    for (entity, position) in query.iter() {
        if position.0.y < DESPAWN_DEPTH {
            commands.entity(entity).despawn();
            println!("Despawned entity {} that fell off the map", entity);
        }
    }
}
fn shmoop_dragging_system(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Query<&GlobalTransform, With<CanBeDraggedOn>>,
    windows: Query<&Window>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut shmoop_query: Query<(&Position, &mut LinearVelocity), (With<Shmoop>, With<Picked>)>,
) {
    let Ok(windows) = windows.single() else {
        return;
    };

    let (camera, camera_transform) = *camera_query;

    let Some(cursor_position) = windows.cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        100.0,
        false,
        &SpatialQueryFilter::DEFAULT,
    ) else {
        return;
    };

    if !ground.contains(hit.entity) {
        return;
    }

    const DRAGGING_SPEED: f32 = 50.0;
    for (position, mut linear_velocity) in shmoop_query.iter_mut() {
        let direction = (ray.origin + (ray.direction * hit.distance)) - position.0;
        if direction.length() > 0.3 {
            let direction = direction.normalize_or_zero() * time.delta_secs();
            linear_velocity.0.x = direction.x * DRAGGING_SPEED;
            linear_velocity.0.y = 0.0;
            linear_velocity.0.z = direction.z * DRAGGING_SPEED;
        } else {
            linear_velocity.0 = Vec3::ZERO;
        }
    }
}

fn destination_abandoning_system(
    mut commands: Commands,
    query: Query<(Entity, &DestinationTime), (With<Shmoop>, Without<Picked>)>,
) {
    for (entity, destination_time) in query.iter() {
        if destination_time.time < 10.0 {
            continue;
        }

        println!("Shmoop {} destination time exceeded", entity);
        commands.entity(entity).remove::<DestinationTime>();
        commands.entity(entity).remove::<ShmoopDestination>();
        commands.entity(entity).remove::<ShmoopInteractionTarget>();
    }
}

fn destination_time_system(time: Res<Time>, mut query: Query<&mut DestinationTime>) {
    for mut destination_time in query.iter_mut() {
        destination_time.time += time.delta_secs();
    }
}
fn shmoop_destination_selection_system(
    map_bounds: Res<MapBounds>,
    mut commands: Commands,
    mut query: Query<
        Entity,
        (
            With<Shmoop>,
            Without<ShmoopDestination>,
            Without<ShmoopInteractionTarget>,
            Without<Picked>,
        ),
    >,
) {
    let target_bounds = map_bounds.half_size * 2.0;
    for entity in query.iter_mut() {
        // info!("ASdd {}", random_range(-target_bounds.x..target_bounds.x));

        let destination = ShmoopDestination {
            target: Vec3::new(
                random_range(-target_bounds.x..target_bounds.x),
                0.0,
                random_range(-target_bounds.z..target_bounds.z),
            ),
        };
        commands
            .entity(entity)
            .insert((destination.clone(), DestinationTime { time: 0.0 }));

        println!(
            "Shmoop {} target: {} {} {}",
            entity, destination.target.x, destination.target.y, destination.target.z,
        );
    }
}

fn select_system(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    spatial_query: SpatialQuery,
    mut commands: Commands,
    mut shmoop_query: Query<
        (Entity, &mut OutlineVolume, Option<&Picked>),
        (
            With<Shmoop>,
            Without<Ground>,
            Without<ShipFloor>,
            Without<Interactable>,
        ),
    >,
    mut interactables_query: Query<
        (Entity, &mut OutlineVolume, &Position),
        (
            With<Interactable>,
            Without<Ground>,
            Without<ShipFloor>,
            Without<Shmoop>,
        ),
    >,
    ship_floor: Query<
        Entity,
        (
            With<ShipFloor>,
            Without<Ground>,
            Without<Shmoop>,
            Without<Interactable>,
        ),
    >,
    ground: Query<
        Entity,
        (
            With<Ground>,
            Without<ShipFloor>,
            Without<Shmoop>,
            Without<Interactable>,
        ),
    >,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let pick = buttons.pressed(PICK_MOUSE_BUTTON);
    let mut picked_entity: Option<Entity> = None;
    {
        for (entity, mut outline_volume, picked) in shmoop_query.iter_mut() {
            if !picked.is_some() {
                outline_volume.visible = false;
                continue;
            }

            picked_entity = Some(entity);

            if !pick {
                outline_volume.visible = false;
                commands.entity(entity).remove::<Picked>();
                println!("Shmoop {} unselected", entity);
            }
        }
    }

    for (_entity, mut outline_volume, _position) in interactables_query.iter_mut() {
        outline_volume.visible = false;
    }

    println!("asd");
    let Ok(windows) = windows.single() else {
        return;
    };
    println!("qwe");

    let (camera, camera_transform) = *camera_query;

    let Some(cursor_position) = windows.cursor_position() else {
        return;
    };

    println!("zxc");
    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        100.0,
        false,
        &SpatialQueryFilter::DEFAULT,
    ) else {
        return;
    };
    println!("Hit entity: {}", hit.entity);

    if let Ok((entity, mut outline_volume, picked)) = shmoop_query.get_mut(hit.entity) {
        outline_volume.visible = true;

        if !pick {
            outline_volume.colour = HOVER_COLOR;
        } else if !picked.is_some() && picked_entity.is_none() {
            outline_volume.colour = PICKING_COLOR;
            commands.entity(entity).insert(Picked);
            commands.entity(entity).remove::<ShmoopDestination>();
            commands.entity(entity).remove::<DestinationTime>();
            picked_entity = Some(entity);
            println!("Shmoop {} selected", entity);
        }
    }

    if let Some(picked_entity) = picked_entity {
        if let Ok((entity, mut outline_volume, position)) = interactables_query.get_mut(hit.entity)
        {
            if pick {
                outline_volume.visible = true;
                outline_volume.colour = TARGET_SELECTION_COLOR;
            } else {
                commands.entity(picked_entity).insert((
                    ShmoopInteractionTarget { entity },
                    ShmoopDestination { target: position.0 },
                    DestinationTime { time: 0.0 },
                ));
            }
        } else if ground.contains(hit.entity) || ship_floor.contains(hit.entity) {
            commands.entity(picked_entity).insert((
                ShmoopDestination {
                    target: ray.origin + (ray.direction * hit.distance),
                },
                DestinationTime { time: 0.0 },
            ));
        }
    }
}
