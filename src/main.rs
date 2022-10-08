use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use rand::Rng;

pub mod player;
pub mod projectile;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec3::ZERO, // disable gravity at all
            ..default()
        })
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(projectile::ProjectilePlugin)
        .add_plugin(player::PlayerPlugin)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(5.0))
                .with_system(spawn_baloon),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // root UI node that covers all screen
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center, // vertical alignment
                justify_content: JustifyContent::Center, // horizontal alignment
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(40.0), Val::Px(40.0)),
                    ..default()
                },
                image: asset_server.load("UI/aim.png").into(),
                ..default()
            });
        })
        .insert(Name::new("UI"));

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
        .insert(Name::new("Spaceship"));

    // Create a sky
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                // We make the dimensions negative because we want to invert the direction
                // of light the mesh diffuses (invert the normals).
                radius: -250.0,
                ..default()
            })),
            // We make the mesh as rough as possible to avoid metallic-like reflections
            material: materials.add(StandardMaterial {
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                emissive: Color::rgb(0.0, 0.05, 0.5),
                ..default()
            }),
            ..default()
        })
        .insert(Name::new("Sky"));

    //Create a ground
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 200.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(Collider::halfspace(Vec3::Y).unwrap())
        .insert(Restitution::coefficient(1.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -3.0, 0.0)))
        .insert(Name::new("Ground"));

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
) {
    let position = loop {
        let position = Vec3 {
            x: rand::thread_rng().gen_range(-100.0..100.0),
            z: rand::thread_rng().gen_range(-100.0..100.0),
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
            linvel: Vec3::Y * rand::thread_rng().gen_range(1.0..5.0),
            angvel: Vec3::Y * rand::thread_rng().gen_range(-2.0..2.0),
            ..default()
        })
        .insert(Collider::ball(radius))
        .insert(RigidBody::Dynamic)
        .insert(projectile::Lifetime(60.0))
        .insert(projectile::ExplosionEffect::Debug)
        .insert(Name::new("Shooting target"));
}
