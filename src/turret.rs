use bevy::prelude::*;

use crate::{
    aiming, collider_setup, gun, projectile::HitPoints, scene_setup::SetupRequired, weapon,
};

/// Emit this event to spawn a turret with specified parameters
pub struct SpawnTurretEvent {
    pub transform: Transform,
    /// Rotation speed in rad/s
    pub rotation_speed: f32,
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
    gun_layer: aiming::GunLayer,
    joints: TurretJoints,
}

impl TurretBundle {
    fn new(joints: Vec<Entity>) -> Self {
        Self {
            gun_layer: aiming::GunLayer::default(),
            joints: TurretJoints(joints),
        }
    }
}

#[derive(Resource)]
struct TurretScene(Handle<Scene>);

fn load_turret_resources(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(TurretScene(assets.load("models/turret.glb#Scene0")));
}

#[derive(Component)]
struct TurretBody;

fn spawn_turret(
    mut commands: Commands,
    turret_scene: Res<TurretScene>,
    mut ev_spawn_turret: EventReader<SpawnTurretEvent>,
) {
    for ev in ev_spawn_turret.iter() {
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
                    .iter()
                    // We are interested only in entities that have Name component
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
                            head = Some(entity);
                        }
                    });

                if let Some(body) = body {
                    commands
                        .entity(body)
                        .insert(TurretBody)
                        .insert(HitPoints::new(200))
                        .insert(collider_setup::ConvexHull::new(collider_parts));
                };

                if let Some(head) = head {
                    commands
                        .entity(head)
                        .insert(TurretBundle::new(joints))
                        .insert(weapon::FlakCannon::new(barrels, 5.0));
                }
            }))
            .insert(Name::new("Turret"));
    }
}

fn orientation(
    turrets: Query<(&aiming::GunLayer, &TurretJoints)>,
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

fn fire_control(mut turrets: Query<(&aiming::GunLayer, &mut gun::Trigger)>) {
    for (gun_layer, mut gun_trigger) in turrets.iter_mut() {
        let threshold = if gun_layer.distance > 100.0 {
            // let's say for simplicity that target is 10m size
            10.0 / gun_layer.distance
        } else {
            0.3
        };
        if gun_layer.distance != 0.0 && gun_layer.angle < threshold {
            gun_trigger.pull();
        }
    }
}

pub struct TurretPlugin;
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_turret_resources)
            .add_event::<SpawnTurretEvent>()
            .add_system(spawn_turret)
            //.add_system(orientation.after(targeting::gun_layer))
            .add_system(orientation.after(aiming::gun_layer))
            .add_system(fire_control.after(orientation));
    }
}
