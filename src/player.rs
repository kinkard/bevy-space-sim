use bevy::{input::mouse::MouseWheel, pbr::wireframe, prelude::*, render::camera};
use bevy_rapier3d::prelude::*;

use crate::{gun, projectile::HitPoints, weapon};

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PrimaryWeapon;

#[derive(Component)]
struct SecondaryWeapon;

fn setup_player(mut commands: Commands) {
    // Create a player entity with a camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        })
        .insert(Player)
        .insert(Name::new("Player"))
        .with_children(|parent| {
            let rate_of_fire = 6.7;
            parent.spawn((
                PrimaryWeapon,
                weapon::MachineGun::new(rate_of_fire),
                TransformBundle::from(Transform::from_translation(-Vec3::Z + 0.2 * Vec3::X)),
            ));
            parent.spawn((
                PrimaryWeapon,
                weapon::MachineGun::new(rate_of_fire),
                TransformBundle::from(Transform::from_translation(-Vec3::Z - 0.2 * Vec3::X)),
            ));
            parent.spawn((
                PrimaryWeapon,
                weapon::MachineGun::new(rate_of_fire),
                TransformBundle::from(Transform::from_translation(-Vec3::Z - 0.2 * Vec3::Y)),
            ));

            parent.spawn((
                SecondaryWeapon,
                weapon::RocketLauncher::new(rate_of_fire),
                TransformBundle::from(Transform::from_translation(-Vec3::Z)),
            ));
        });
}

#[derive(Component)]
struct ConsoleText;

fn setup_hud(mut commands: Commands, assets: Res<AssetServer>) {
    // root UI node that covers all screen
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center, // vertical alignment
                justify_content: JustifyContent::Center, // horizontal alignment
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            // Aim in the middle of the screen
            parent.spawn(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(40.0), Val::Px(40.0)),
                    ..default()
                },
                image: assets.load("UI/aim.png").into(),
                ..default()
            });

            // Semi-transparent section in the left bottom corner for in-game infromation
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(25.0), Val::Percent(25.0)),
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            right: Val::Px(10.0),
                            bottom: Val::Px(10.0),
                            ..default()
                        },
                        align_items: AlignItems::FlexStart, // vertical alignment to the top
                        justify_content: JustifyContent::FlexStart, // horizontal alignment to the left
                        padding: UiRect::all(Val::Px(5.0)),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    },
                    background_color: Color::rgba(0.7, 0.7, 0.7, 0.3).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "",
                            TextStyle {
                                font: assets.load("fonts/FiraMono-Medium.ttf"),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        ))
                        .insert(ConsoleText);
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
    if keys.just_released(KeyCode::Space) {
        *mouse_guidance = !*mouse_guidance;
    }

    let click_guidance = !egui.ctx_mut().is_using_pointer() && mouse.pressed(MouseButton::Left);
    if *mouse_guidance || click_guidance {
        let window = windows.primary_mut();
        // egui sets it's own icon, so we override cursor it on every frame
        window.set_cursor_icon(if *mouse_guidance {
            CursorIcon::Crosshair
        } else {
            CursorIcon::Default
        });

        if let Some(pos) = window.cursor_position() {
            let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
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

fn zoom_camera(
    mut scroll: EventReader<MouseWheel>,
    mut projection: Query<&mut camera::Projection, With<Camera3d>>,
    mut egui: ResMut<bevy_inspector_egui::bevy_egui::EguiContext>,
) {
    if egui.ctx_mut().wants_pointer_input() {
        return;
    }

    let delta_zoom: f32 = scroll.iter().map(|e| e.y).sum();
    if delta_zoom == 0.0 {
        return;
    }

    if let Ok(mut projection) = projection.get_single_mut() {
        if let camera::Projection::Perspective(projection) = projection.as_mut() {
            projection.fov = (projection.fov - delta_zoom * 0.001)
                // restrict FOV
                .clamp(std::f32::consts::PI / 32.0, std::f32::consts::FRAC_PI_4);
        }
    }
}

fn primary_weapon_shoot(
    keys: Res<Input<KeyCode>>,
    mut triggers: Query<&mut gun::Trigger, With<PrimaryWeapon>>,
) {
    if keys.pressed(KeyCode::LAlt) {
        for mut trigger in triggers.iter_mut() {
            trigger.pull();
        }
    }
}

fn secondary_weapon_shoot(
    keys: Res<Input<KeyCode>>,
    mut triggers: Query<&mut gun::Trigger, With<SecondaryWeapon>>,
) {
    if keys.pressed(KeyCode::LControl) {
        for mut trigger in triggers.iter_mut() {
            trigger.pull();
        }
    }
}

/// Annotates current locked target.
#[derive(Component)]
pub struct LockedTarget;

fn select_target(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    camera: Query<&Transform, With<Camera>>,
    targets: Query<Entity, With<LockedTarget>>,
    children: Query<&Children>,
    with_mesh: Query<&Handle<Mesh>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::T) {
        let transform = camera.single();
        if let Some((entity, _)) = rapier_context.cast_ray(
            transform.translation,
            transform.forward(),
            Real::MAX,
            false,
            QueryFilter::default(),
        ) {
            fn iter_hierarchy(
                entity: Entity,
                children_query: &Query<&Children>,
                f: &mut impl FnMut(Entity),
            ) {
                (f)(entity);
                if let Ok(children) = children_query.get(entity) {
                    for child in children.iter().copied() {
                        iter_hierarchy(child, children_query, f);
                    }
                }
            }

            // Select a new target and highlight it via Wireframe
            if !targets.contains(entity) {
                commands.entity(entity).insert(LockedTarget);
                iter_hierarchy(entity, &children, &mut |entity| {
                    if with_mesh.contains(entity) {
                        commands.entity(entity).insert(wireframe::Wireframe);
                    }
                });
            }

            // Remove previous target selection if any.
            // This order also unselects previous target on a repeated select.
            for prev_target in targets.iter() {
                commands.entity(prev_target).remove::<LockedTarget>();
                iter_hierarchy(prev_target, &children, &mut |entity| {
                    commands.entity(entity).remove::<wireframe::Wireframe>();
                });
            }
        }
    }
}

fn show_selected_target_info(
    player: Query<&GlobalTransform, With<Player>>,
    target: Query<(Option<&Name>, &GlobalTransform, Option<&HitPoints>), With<LockedTarget>>,
    mut console: Query<&mut Text, With<ConsoleText>>,
) {
    let mut console = console.single_mut();
    if let Ok((name, transform, hp)) = target.get_single() {
        let player_pos = player.single().translation();
        let distance = player_pos.distance(transform.translation());

        let name = name.map_or("-- Unknown --", |name| name.as_str());
        console.sections[0].value = format!("Selected: {name}\nDistance to target: {distance:.2}m");

        if let Some(hp) = hp {
            console.sections[0].value += &format!("\nHit Points: {}%", hp.percent());
        }
    } else {
        console.sections[0].value = String::from("Press 'T' to select a target.");
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_startup_system(setup_hud)
            .add_plugin(wireframe::WireframePlugin)
            .add_system(select_target)
            .add_system(show_selected_target_info)
            .add_system(move_player)
            .add_system(zoom_camera)
            .add_system(primary_weapon_shoot)
            .add_system(secondary_weapon_shoot);
    }
}
