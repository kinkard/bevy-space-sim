use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    collider_setup, gun, player, projectile::HitPoints, scene_setup::SetupRequired, weapon,
};

/// Emit this event to create a turret with specified parameters
pub struct CreateTurretEvent {
    pub transform: Transform,
    /// Rotation speed in rad/s
    pub rotation_speed: f32,
}

/// Turret's locked target.
#[derive(Component, Default)]
struct LockedTarget(Option<Entity>);

/// Turret's white list that should be never targeted
#[derive(Component, Default)]
struct IgnoreTargets(Vec<Entity>);

/// Annotates an entity to be used for building direction vector to the specified target.
/// Turret orientation system rotates joints (entities with `Joint` component) to
/// orient entity with `GunLayer` component towards specified target if provided.
#[derive(Component, Default)]
struct GunLayer {
    axis: Vec3,
    angle: f32,
}

/// Links turret main entity with joints that will be used for turret orientation.
/// This component should be assigned to the same entity that contains `GunLayer` component.
/// Linked entities should have `Joint` component.
#[derive(Component)]
struct TurretJoints(Vec<Entity>);

/// Annotates rotational turret joint.
/// Due to strange magic inside scene setup from GLTF, the only axis that can be rotated without artifacts is Y.
/// Which means that joint's parent's Y axis should oriented in the direction of the intended joint's rotation.
/// In other words, Joint always rotates around parent's Y.
#[derive(Component)]
struct Joint {
    rotation_speed: f32,
}

#[derive(Bundle)]
struct TurretBundle {
    target: LockedTarget,
    gun_layer: GunLayer,
    joints: TurretJoints,
}

impl TurretBundle {
    fn new(joints: Vec<Entity>) -> Self {
        Self {
            target: LockedTarget::default(),
            gun_layer: GunLayer::default(),
            joints: TurretJoints(joints),
        }
    }
}

#[derive(Resource)]
struct TurretScene(Handle<Scene>);

fn load_turret_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(TurretScene(assets.load("models/turret.glb#Scene0")));
}

fn create_turret(
    mut commands: Commands,
    turret_scene: Res<TurretScene>,
    mut ev_create_turret: EventReader<CreateTurretEvent>,
) {
    for ev in ev_create_turret.iter() {
        let rotation_speed = ev.rotation_speed;
        commands
            .spawn(SceneBundle {
                scene: turret_scene.0.clone(),
                transform: ev.transform,
                ..default()
            })
            .insert(SetupRequired::new(move |commands, entities| {
                let mut collider_parts = vec![];
                let mut joints = vec![];
                let mut barrels = vec![];

                let mut head: Option<Entity> = None;
                let mut body: Option<Entity> = None;

                entities
                    // Skip entities with `Handle<Mesh>` as we should operate only with GLTF's Nodes
                    .filter(|e| !e.contains::<Handle<Mesh>>())
                    .filter_map(|e| e.get::<Name>().map(|name| (e.id(), name)))
                    .for_each(|(entity, name)| {
                        if name.starts_with("Muzzle") {
                            commands.entity(entity).insert(gun::Barrel);
                            barrels.push(entity);
                        } else if name.starts_with("Body") {
                            commands.entity(entity).insert(Joint { rotation_speed });
                            joints.push(entity);
                            collider_parts.push(entity);
                            body = Some(entity);
                        } else if name.starts_with("Head") {
                            commands.entity(entity).insert(Joint { rotation_speed });
                            joints.push(entity);
                            collider_parts.push(entity);
                            head = Some(entity);
                        }
                    });

                let mut ignore_targets = vec![];
                if let Some(body) = body {
                    commands
                        .entity(body)
                        .insert(HitPoints::new(200))
                        .insert(collider_setup::ConvexHull::new(collider_parts));
                    ignore_targets.push(body);
                };

                if let Some(head) = head {
                    commands
                        .entity(head)
                        .insert(TurretBundle::new(joints))
                        .insert(weapon::FlakCannon::new(barrels, 5.0));

                    if !ignore_targets.is_empty() {
                        commands.entity(head).insert(IgnoreTargets(ignore_targets));
                    }
                }
            }))
            .insert(Name::new("Turret"));
    }
}

fn select_target(
    target: Query<Entity, With<player::LockedTarget>>,
    mut turrets: Query<(&mut LockedTarget, Option<&IgnoreTargets>)>,
) {
    let Some(target) = target.iter().next() else {
        return; // nothing to do
    };

    for (mut turret, ignore) in turrets.iter_mut() {
        if !matches!(ignore, Some(ignore) if ignore.0.contains(&target)) {
            turret.0 = Some(target);
        }
    }
}

fn gun_layer(
    mut turrets: Query<(
        &GlobalTransform,
        &LockedTarget,
        &mut GunLayer,
        &mut gun::Trigger,
    )>,
    targets: Query<(&GlobalTransform, Option<&Velocity>)>,
) {
    for (turret, target, mut gun_layer, mut gun_trigger) in turrets.iter_mut() {
        let Some((target, velocity)) = target.0.and_then(|e| targets.get(e).ok()) else {
            // Target is not selected or not exists anymore - nothing to do.
            // TODO: implement turret parking in a default position after some delay
            gun_layer.angle = 0.0;
            continue;
        };

        // Adjust target position to compensate its velocity if any
        let target_pos = if let Some(velocity) = velocity {
            let projectile_speed = 100.0;
            let target_pos = target.translation();
            let time = target_pos.distance(turret.translation()) / projectile_speed;
            target_pos + velocity.linvel * time
        } else {
            target.translation()
        };

        let to_target = target_pos - turret.translation();
        let distance = to_target.length();
        // Required rotation to orient turret towards `target_pos`
        (gun_layer.axis, gun_layer.angle) =
            Quat::from_rotation_arc(turret.forward(), to_target * distance.recip()).to_axis_angle();

        let threshold = if distance > 100.0 {
            // let's say for simplicity that target is 10m size
            10.0 / distance
        } else {
            0.3
        };
        if gun_layer.angle < threshold {
            gun_trigger.pull();
        }
    }
}

fn orientation(
    turrets: Query<(&GunLayer, &TurretJoints)>,
    transforms: Query<&GlobalTransform, With<Children>>,
    time: Res<Time>,
    mut joints: Query<(&mut Transform, &Parent, &Joint)>,
) {
    for (gun_layer, turret_joints) in turrets.iter() {
        if gun_layer.angle == 0.0 {
            continue;
        }

        for joint in turret_joints.0.iter() {
            let (mut joint, parent, cfg) = joints.get_mut(*joint).unwrap();

            // As was mentioned in the `Joint` doc, they rotates around parent's Y axis
            let pivot = transforms.get(parent.get()).unwrap().up();

            joint.rotate_y((pivot.dot(gun_layer.axis) * gun_layer.angle).clamp(
                -cfg.rotation_speed * time.delta_seconds(),
                cfg.rotation_speed * time.delta_seconds(),
            ));
        }
    }
}

pub struct TurretPlugin;
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_turret_scene)
            .add_event::<CreateTurretEvent>()
            .add_system(create_turret)
            .add_system(select_target)
            .add_system(gun_layer)
            .add_system(orientation.after(gun_layer));
    }
}
