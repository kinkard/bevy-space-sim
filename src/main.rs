use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec3::ZERO, // disable gravity at all
            ..default()
        })
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(move_camera)
        .add_system(spawn_projectile)
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
        });

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
        .insert(Restitution::coefficient(1.0));

    // Create a sky
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            // We make the dimensions negative because we want to invert the direction
            // of light the mesh diffuses (invert the normals).
            radius: -150.0,
            depth: -1.0,
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
    });

    //Create a ground
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 200.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(Restitution::coefficient(1.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    // Create a light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 4000.0,
            range: 100.0,
            radius: 2.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        ..default()
    });

    // Create a camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 4.0, 10.0),
        ..default()
    });
}

fn move_camera(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>,
    mut mouse_guidance: Local<bool>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut camera_speed = 10.0;
    if keys.pressed(KeyCode::LShift) {
        camera_speed *= 3.0;
    }
    let camepa_step = camera_speed * time.delta_seconds();

    let mut translation = Vec3::ZERO;
    if keys.pressed(KeyCode::W) {
        // strafe up
        translation.y += camepa_step;
    }
    if keys.pressed(KeyCode::S) {
        // strafe down
        translation.y -= camepa_step;
    }
    if keys.pressed(KeyCode::A) {
        // strafe right
        translation.x -= camepa_step;
    }
    if keys.pressed(KeyCode::D) {
        // strafe left
        translation.x += camepa_step;
    }
    if keys.pressed(KeyCode::X) {
        // move forward
        translation.z -= camepa_step;
    }
    if keys.pressed(KeyCode::Z) {
        // move backward
        translation.z += camepa_step;
    }

    let mut rotation = Quat::IDENTITY;
    if keys.pressed(KeyCode::Q) {
        // rotate counter clockwise
        rotation *= Quat::from_rotation_z(camepa_step * 10.0_f32.to_radians());
    }
    if keys.pressed(KeyCode::E) {
        // rotate counter clockwise
        rotation *= Quat::from_rotation_z(camepa_step * -10.0_f32.to_radians());
    }

    // Enable mouse guidance if Space is pressed
    let window = windows.primary_mut();
    if keys.just_released(KeyCode::Space) {
        *mouse_guidance = !*mouse_guidance;
        let icon = if *mouse_guidance {
            CursorIcon::Crosshair
        } else {
            CursorIcon::Default
        };
        window.set_cursor_icon(icon);
    }

    if *mouse_guidance || mouse.pressed(MouseButton::Left) {
        let center = Vec2 {
            x: window.width() / 2.0,
            y: window.height() / 2.0,
        };

        if let Some(pos) = window.cursor_position() {
            let offset = center - pos;
            // Safe zone around screen center
            if offset.length_squared() > 400.0 {
                rotation *= Quat::from_rotation_y(0.005 * offset.x.to_radians());
                rotation *= Quat::from_rotation_x(-0.005 * offset.y.to_radians());
            }
        }
    }

    for mut transform in query.iter_mut() {
        transform.rotate_local(rotation);
        translation = transform.rotation * translation;
        transform.translation += translation;
    }
}

fn spawn_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<Input<KeyCode>>,
    query: Query<&mut Transform, With<Camera3d>>,
) {
    if keys.just_pressed(KeyCode::LControl) {
        // get came transform to spawn rocket in a right direction
        if let Some(transform) = query.iter().next() {
            // spawn in a front of the camera
            let position = transform.translation + (transform.rotation * (-1.0 * Vec3::Z));
            // velocity in a camera direction
            let velocity = transform.rotation * -Vec3::Z * 20.0;

            // Create a small bouncing ball
            let radius = 0.1;
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius,
                        sectors: 64,
                        stacks: 32,
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(1.0, 0.5, 0.5),
                        unlit: true,
                        ..default()
                    }),
                    ..default()
                })
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(radius))
                .insert(Restitution::coefficient(0.7))
                .insert_bundle(TransformBundle::from(Transform::from_translation(position)))
                .insert(Velocity {
                    linvel: velocity,
                    ..default()
                })
                .with_children(|children| {
                    children.spawn_bundle(PointLightBundle {
                        point_light: PointLight {
                            intensity: 1500.0,
                            radius,
                            color: Color::rgb(1.0, 0.2, 0.2),
                            ..default()
                        },
                        ..default()
                    });
                });
        }
    }
}
