use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy::time::FixedTimestep;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use rand::Rng;

pub mod aiming;
pub mod collider_setup;
pub mod drone;
pub mod gun;
pub mod player;
pub mod projectile;
pub mod scene_setup;
pub mod skybox;
pub mod turret;
pub mod weapon;

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
        .add_plugin(collider_setup::ColliderSetupPlugin)
        .add_plugin(skybox::SkyboxPlugin)
        .add_plugin(projectile::ProjectilePlugin)
        .add_plugin(aiming::AimingPlugin)
        .add_plugin(gun::GunPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(turret::TurretPlugin)
        .add_plugin(drone::DronePlugin)
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
    mut ev_spawn_turret: EventWriter<turret::SpawnTurretEvent>,
    mut ev_spawn_drone: EventWriter<drone::SpawnDroneEvent>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/spaceship_v1.glb#Scene0"),
            ..default()
        })
        .insert(Restitution::coefficient(1.0))
        .insert(TransformBundle::from(Transform::from_scale(
            2.0 * Vec3::ONE, // adjust model size for realizm
        )))
        .insert(scene_setup::SetupRequired::new(
            move |commands, entities| {
                let mut root: Option<Entity> = None;
                let mut mesh_source: Option<Entity> = None;
                for entity in entities {
                    if entity.contains::<SceneInstance>() {
                        root = Some(entity.id());
                    }
                    if entity.contains::<Handle<Mesh>>() {
                        mesh_source = Some(entity.id());
                    }
                }

                commands
                    .entity(root.unwrap())
                    .insert(collider_setup::ConvexDecomposition {
                        mesh_source: mesh_source.unwrap(),
                        parameters: VHACDParameters {
                            concavity: 0.06,
                            ..default()
                        },
                    });
            },
        ))
        .insert(projectile::HitPoints::new(2000))
        .insert(Name::new("Spaceship"));

    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/artillery_platform.glb#Scene0"),
            ..default()
        })
        .insert(Restitution::coefficient(1.0))
        .insert(RigidBody::Dynamic)
        .insert(TransformBundle::from(Transform {
            translation: Vec3::new(0.0, 100.0, -300.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            scale: Vec3::splat(2.0),
            ..default()
        }))
        .insert(scene_setup::SetupRequired::new(
            move |commands, entities| {
                let collider_parts: Vec<_> = entities
                    .iter()
                    .filter(|entity| entity.contains::<Handle<Mesh>>())
                    .map(|entity| entity.id())
                    .collect();

                let mut root_entity = None;
                let mut sphere = None;
                for entity in entities {
                    if entity.contains::<SceneInstance>() {
                        root_entity = Some(entity.id());
                    }
                    if matches!(entity.get::<Name>(), Some(name) if name.starts_with("Sphere")) {
                        sphere = Some(entity.id());
                    }
                }

                commands
                    .entity(root_entity.unwrap())
                    .insert(collider_setup::ConvexHull::new(collider_parts));
                commands.entity(sphere.unwrap()).add_children(|children| {
                    children.spawn(PointLightBundle {
                        point_light: PointLight {
                            intensity: 30000.0,
                            radius: 0.1,
                            color: Color::rgb(0.2, 0.2, 1.0),
                            shadows_enabled: true,
                            ..default()
                        },
                        ..default()
                    });
                });
            },
        ))
        .insert(projectile::HitPoints::new(2000))
        .insert(Name::new("Artillery Platform"));

    for (drone, position) in [
        (drone::Drone::Infiltrator, Vec3::new(-1600.0, 10.0, 0.0)),
        (drone::Drone::Infiltrator, Vec3::new(-1500.0, 10.0, 50.0)),
        (drone::Drone::Infiltrator, Vec3::new(-1600.0, 10.0, 100.0)),
        (drone::Drone::Praetor, Vec3::new(1600.0, 10.0, 100.0)),
        (drone::Drone::Praetor, Vec3::new(1500.0, 10.0, 50.0)),
        (drone::Drone::Praetor, Vec3::new(1600.0, 10.0, 0.0)),
    ] {
        ev_spawn_drone.send(drone::SpawnDroneEvent {
            drone,
            transform: Transform::from_translation(position),
        });
    }

    let pos = 25.0;
    for (x, z) in [(-pos, -pos), (pos, -pos), (-pos, pos), (pos, pos)] {
        ev_spawn_turret.send(turret::SpawnTurretEvent {
            transform: Transform::from_translation(Vec3::new(x, -3.0, z)),
            rotation_speed: 120_f32.to_radians(),
        });
    }

    // Create a light
    commands.spawn(PointLightBundle {
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
        .spawn(PbrBundle {
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
