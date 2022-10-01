use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_system(move_camera)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = true;

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

    // Create a sky
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            // We make the dimensions negative because we want to invert the direction
            // of light the mesh diffuses (invert the normals).
            radius: -150.0,
            depth: -1.0,
            ..Default::default()
        })),
        // We make the mesh as rough as possible to avoid metallic-like reflections
        material: materials.add(StandardMaterial {
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            emissive: Color::rgb(0.0, 0.05, 0.5),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        ..Default::default()
    });

    //Create a ground
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 150.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
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
        ..Default::default()
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
