use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::{
    player::{LockedTarget, PrimaryWeapon}, // todo: replace by own
    scene_setup::SetupRequired,
};

/// Emit this event to create a turret with specified Transform.
pub struct CreateTurretEvent(pub Transform);

/// Annotates an entity to be used for building direction vector to the specified target.
/// Turret orientation system rotates joints (entities with `Joint` component) to
/// orient entity with `GunLayer` component towards specified target if provided.
/// Should be in the same entity that contains `TurretJoints` component.
#[derive(Component, Default)]
struct GunLayer(Option<Entity>);

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
struct Joint;

#[derive(Bundle)]
struct TurretBundle {
    gun_layer: GunLayer,
    joints: TurretJoints,
}

impl TurretBundle {
    fn new(joints: Vec<Entity>) -> Self {
        Self {
            gun_layer: GunLayer(None),
            joints: TurretJoints(joints),
        }
    }
}

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
        commands
            .spawn_bundle(SceneBundle {
                scene: turret_scene.0.clone(),
                transform: ev.0,
                ..default()
            })
            .insert(SetupRequired::new(|commands, entities| {
                let mut joints = vec![];
                entities
                    // Skip entities with `Handle<Mesh>` as we should operate only with GLTF's Nodes
                    .filter(|e| e.get::<Handle<Mesh>>().is_none())
                    .filter_map(|e| e.get::<Name>().map(|name| (e.id(), name)))
                    // fold() allows us to find "Head" node and insert all joints into it once all of them are found
                    .fold(None, |head, (entity, name)| {
                        if name.starts_with("Muzzle") {
                            commands.entity(entity).insert(PrimaryWeapon);
                            head
                        } else if name.starts_with("Body") {
                            commands.entity(entity).insert(Joint);
                            joints.push(entity);
                            head
                        } else if name.starts_with("Head") {
                            commands.entity(entity).insert(Joint);
                            joints.push(entity);
                            Some(entity) // set "Head" entity
                        } else {
                            head
                        }
                    })
                    .and_then(|head| {
                        commands
                            .entity(head)
                            .insert_bundle(TurretBundle::new(joints));
                        Some(head)
                    });
            }))
            .insert(Name::new("Turret"));
    }
}

fn dispatch_targets(target: Query<Entity, With<LockedTarget>>, mut turrets: Query<&mut GunLayer>) {
    let Ok(target) = target.get_single() else {
        return; // nothing to do
    };

    for mut turret in turrets.iter_mut() {
        turret.0 = Some(target);
    }
}

fn turret_oritentation(
    turrets: Query<(&GlobalTransform, &GunLayer, &TurretJoints)>,
    transforms: Query<(&GlobalTransform, Option<&Velocity>)>,
    mut joints: Query<(&mut Transform, &Parent), With<Joint>>,
) {
    for (turret, target, turret_joints) in turrets.iter() {
        let Some((target, velocity)) = target.0.and_then(|e| transforms.get(e).ok()) else {
            // Target is not selected or not exists anymore - nothing to do.
            // TODO: implement turret parking in a default position after some delay
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

        // Required rotation to orient turret towards `target_pos`
        let (axis, angle) = Quat::from_rotation_arc(
            turret.forward(),
            (target_pos - turret.translation()).normalize(),
        )
        .to_axis_angle();

        for joint in turret_joints.0.iter() {
            let (mut joint, parent) = joints.get_mut(*joint).unwrap();
            // As was mentioned in the `Joint` doc, they rotates around parent's Y axis
            let pivot = if let Ok((parent, _)) = transforms.get(parent.get()) {
                parent.up()
            } else {
                Vec3::Y
            };
            joint.rotate_y(pivot.dot(axis) * angle);
        }
    }
}

pub struct TurretPlugin;
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_turret_scene)
            .add_event::<CreateTurretEvent>()
            .add_system(create_turret)
            .add_system(dispatch_targets)
            .add_system(turret_oritentation);
    }
}
