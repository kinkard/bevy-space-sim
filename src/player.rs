use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::projectile;

#[derive(Component)]
struct Player;

#[derive(Default)]
struct WeaponState {
    fire_calldown: Timer,
}

#[derive(Component)]
struct PrimaryWeapon;

#[derive(Component)]
struct SecondaryWeapon;

fn setup_player(mut commands: Commands) {
    // Create a player entity with a camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        })
        .insert(Player)
        .insert(Name::new("Player"))
        .with_children(|parent| {
            parent
                .spawn()
                .insert(PrimaryWeapon)
                .insert_bundle(TransformBundle::from(Transform::from_translation(
                    -Vec3::Z + 0.2 * Vec3::X,
                )));
            parent
                .spawn()
                .insert(PrimaryWeapon)
                .insert_bundle(TransformBundle::from(Transform::from_translation(
                    -Vec3::Z - 0.2 * Vec3::X,
                )));
            parent
                .spawn()
                .insert(PrimaryWeapon)
                .insert_bundle(TransformBundle::from(Transform::from_translation(
                    -Vec3::Z - 0.2 * Vec3::Y,
                )));

            parent
                .spawn()
                .insert(SecondaryWeapon)
                .insert_bundle(TransformBundle::from(Transform::from_translation(-Vec3::Z)));
        });
}

fn setup_hud(mut commands: Commands, assets: Res<AssetServer>) {
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
                image: assets.load("UI/aim.png").into(),
                ..default()
            });
        })
        .insert(Name::new("UI"));
}

fn move_player(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    mut mouse_guidance: Local<bool>,
    mut windows: ResMut<Windows>,
    mut egui: ResMut<bevy_inspector_egui::bevy_egui::EguiContext>,
    mut player_transform: Query<&mut Transform, With<Player>>,
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
    }

    let click_guidance = !egui.ctx_mut().is_using_pointer() && mouse.pressed(MouseButton::Left);
    if *mouse_guidance || click_guidance {
        // egui sets it's own icon, so we override cursor it on every frame
        window.set_cursor_icon(if *mouse_guidance {
            CursorIcon::Crosshair
        } else {
            CursorIcon::Default
        });
        let center = Vec2 {
            x: window.width() / 2.0,
            y: window.height() / 2.0,
        };

        if let Some(pos) = window.cursor_position() {
            let offset = center - pos;
            // Safe zone around screen center for mouse_guidance mode
            if click_guidance || offset.length_squared() > 400.0 {
                rotation *= Quat::from_rotation_y(0.005 * offset.x.to_radians());
                rotation *= Quat::from_rotation_x(-0.005 * offset.y.to_radians());
            }
        }
    }

    let mut transform = player_transform.single_mut();
    transform.rotate_local(rotation);
    translation = transform.rotation * translation;
    transform.translation += translation;
}

fn primary_weapon_shoot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<Input<KeyCode>>,
    query: Query<&mut GlobalTransform, With<PrimaryWeapon>>,
    mut weapon_state: ResMut<WeaponState>,
    time: Res<Time>,
) {
    // TODO: find a better place to update weapon's timers
    weapon_state.fire_calldown.tick(time.delta());

    // Small and fast projectiles, prototype for bullets
    if keys.pressed(KeyCode::LAlt) && weapon_state.fire_calldown.just_finished() {
        for transform in query.iter() {
            // rotate `shape::Capsule` to to align with camera direction
            let capsule_rotation = Quat::from_rotation_x(std::f32::consts::PI * 0.5);

            // Create a small bullet
            let radius = 0.02;
            commands.spawn_bundle(projectile::ProjectileBundle {
                collider: Collider::capsule_y(8.0 * radius, radius),
                mesh_material: PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Capsule {
                        radius,
                        depth: 16.0 * radius,
                        ..default()
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true,
                        // exclude this material from shadows calculations
                        ..default()
                    }),
                    transform: Transform {
                        translation: transform.translation(),
                        rotation: Quat::from_affine3(&transform.affine()) * capsule_rotation,
                        scale: Vec3::ONE,
                    },
                    ..default()
                },
                velocity: Velocity {
                    linvel: transform.forward() * 100.0,
                    ..default()
                },
                lifetime: projectile::Lifetime(10.0),
                explosion: projectile::ExplosionEffect::Small,
                ..default()
            });
        }
    }
}

fn secondary_weapon_shoot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<Input<KeyCode>>,
    query: Query<&mut GlobalTransform, With<SecondaryWeapon>>,
) {
    // big and slow projectile, prototype for rocket
    if keys.just_pressed(KeyCode::LControl) {
        for transform in query.iter() {
            let radius = 0.1;
            commands
                .spawn_bundle(projectile::ProjectileBundle {
                    mesh_material: PbrBundle {
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
                        transform: Transform::from_translation(transform.translation()),
                        ..default()
                    },
                    velocity: Velocity {
                        linvel: transform.forward() * 20.0,
                        ..default()
                    },
                    collider: Collider::ball(radius),
                    lifetime: projectile::Lifetime(30.0),
                    explosion: projectile::ExplosionEffect::Big,
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

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WeaponState {
            fire_calldown: Timer::from_seconds(0.1, true),
        })
        .add_startup_system(setup_player)
        .add_startup_system(setup_hud)
        .add_system(move_player)
        .add_system(primary_weapon_shoot)
        .add_system(secondary_weapon_shoot);
    }
}
