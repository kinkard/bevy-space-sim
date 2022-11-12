use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use rand::Rng;

pub mod player;
pub mod projectile;
pub mod scene_setup;
pub mod skybox;
pub mod turret;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec3::ZERO, // disable gravity at all
            ..default()
        })
        .add_plugin(scene_setup::SceneSetupPlugin)
        .add_plugin(skybox::SkyboxPlugin)
        .add_plugin(projectile::ProjectilePlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(turret::TurretPlugin)
        .add_startup_system(setup_env)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(5.0))
                .with_system(spawn_baloon),
        )
        .insert_resource(Msaa { samples: 4 })
        .add_system(update_msaa)
        .add_system(bevy::window::close_on_esc);

    #[cfg(debug_assertions)]
    app.add_plugin(RapierDebugRenderPlugin::default());

    app.run();
}

fn setup_env(
    mut commands: Commands,
    mut ev_create_turret: EventWriter<turret::CreateTurretEvent>,
    asset_server: Res<AssetServer>,
) {
    // Space ship with a collision model, computed by V-HACD algorithm based on model shape
    // N.B.: Due to async collider loading implementation and it's isolation from bevy,
    // any `TransformBundle` will be applied only on a visual model, but not to the collider.
    // Consider https://github.com/nicopap/bevy-scene-hook to use model's mesh once it is loaded or
    // manually create a `ColliderBuilder::compound` to represent ship's collider.
    let scene = asset_server.load("models/spaceship_v1.glb#Scene0");
    let ship_collider = AsyncSceneCollider {
        handle: scene.clone(),
        shape: Some(ComputedColliderShape::ConvexDecomposition(
            VHACDParameters::default(),
        )),
        named_shapes: bevy::utils::HashMap::default(),
    };
    commands
        .spawn_bundle(SceneBundle { scene, ..default() })
        .insert(ship_collider)
        .insert(Restitution::coefficient(1.0))
        .insert_bundle(TransformBundle::from(Transform::from_scale(
            2.0 * Vec3::ONE, // adjust model size for realizm
        )))
        // todo: resolve how to add hitpoints to the collider entity
        //.insert(projectile::HitPoints::new(1000))
        .insert(Name::new("Spaceship"));

    let pos = 25.0;
    for (x, z, speed) in [
        (-pos, -pos, 30.0_f32),
        (pos, -pos, 90.0_f32),
        (-pos, pos, 180.0_f32),
        (pos, pos, 240.0_f32),
    ] {
        ev_create_turret.send(turret::CreateTurretEvent {
            transform: Transform::from_translation(Vec3::new(x, -3.0, z)),
            rotation_speed: speed.to_radians(),
        });
    }

    // Create a light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 40000.0,
            range: 200.0,
            radius: 20.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 50.0, 0.0),
        ..default()
    });
}

fn spawn_baloon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    mut baloon_number: Local<u32>,
) {
    let mut rng = rand::thread_rng();
    let position = loop {
        let position = Vec3 {
            x: rng.gen_range(-100.0..100.0),
            z: rng.gen_range(-100.0..100.0),
            y: 2.0,
        };
        // Regenerate position if it is inside safe area (where space ship is located)
        if position.x.abs() > 10.0 && position.z.abs() > 10.0 {
            break position;
        }
    };

    let radius = 3.0;
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius,
                sectors: 64,
                stacks: 32,
            })),
            material: materials.add(StandardMaterial {
                base_color_texture: assets.load("textures/aim2.png").into(),
                ..default()
            }),
            transform: Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI * 0.5)),
            ..default()
        })
        .insert(Velocity {
            linvel: Vec3::Y * rng.gen_range(1.0..5.0),
            angvel: Vec3::Y * rng.gen_range(-2.0..2.0),
            ..default()
        })
        .insert(Collider::ball(radius))
        .insert(RigidBody::Dynamic)
        .insert(projectile::Lifetime(60.0))
        .insert(projectile::HitPoints::new(20))
        .insert(Name::new(format!("Shooting target #{}", *baloon_number)));
    *baloon_number += 1;
}

fn update_msaa(keys: Res<Input<KeyCode>>, mut msaa: ResMut<Msaa>) {
    if keys.just_pressed(KeyCode::M) {
        // Unfortunately, WGPU currently only supports 1 or 4 samples.
        // See https://github.com/gfx-rs/wgpu/issues/1832 for more info.
        if msaa.samples == 4 {
            info!("MSAA: disabled");
            msaa.samples = 1;
        } else {
            info!("MSAA: enabled 4x");
            msaa.samples = 4;
        }
    }
}
