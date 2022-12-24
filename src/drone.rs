use bevy::{prelude::*, scene::SceneInstance};
use bevy_rapier3d::prelude::*;
use std::ops::{Index, IndexMut};

use crate::{aiming, collider_setup, gun, projectile, scene_setup, weapon};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Drone {
    /// Drone with 3 guns
    Praetor,
    /// Drone with 2 guns
    Infiltrator,
}

/// Emit this event to spawn a drone with specified parameters
pub struct SpawnDroneEvent {
    pub drone: Drone,
    pub transform: Transform,
}

#[derive(Bundle, Clone, Default)]
struct DroneBundle {
    scene: Handle<Scene>,
    name: Name,
    hitpoints: projectile::HitPoints,
    rotation_speed: RotationSpeed,
}

#[derive(Component)]
struct Guns(Vec<Entity>);

/// Angular velocity limit
#[derive(Component, Clone, Default)]
struct RotationSpeed(f32);

#[derive(Resource, Default)]
struct DroneResources([DroneBundle; 2]);

impl Index<Drone> for DroneResources {
    type Output = DroneBundle;
    fn index(&self, index: Drone) -> &Self::Output {
        match index {
            Drone::Praetor => &self.0[0],
            Drone::Infiltrator => &self.0[1],
        }
    }
}

impl IndexMut<Drone> for DroneResources {
    fn index_mut(&mut self, index: Drone) -> &mut Self::Output {
        match index {
            Drone::Praetor => &mut self.0[0],
            Drone::Infiltrator => &mut self.0[1],
        }
    }
}

fn load_drone_resources(mut commands: Commands, assets: Res<AssetServer>) {
    let mut resources = DroneResources::default();
    resources[Drone::Praetor] = DroneBundle {
        scene: assets.load("models/praetor.glb#Scene0"),
        name: Name::new("Drone::Praetor"),
        hitpoints: projectile::HitPoints::new(300),
        rotation_speed: RotationSpeed(60_f32.to_radians()),
    };
    resources[Drone::Infiltrator] = DroneBundle {
        scene: assets.load("models/infiltrator.glb#Scene0"),
        name: Name::new("Drone::Infiltrator"),
        hitpoints: projectile::HitPoints::new(200),
        rotation_speed: RotationSpeed(90_f32.to_radians()),
    };
    commands.insert_resource(resources);
}

fn spawn_drone(
    mut commands: Commands,
    resources: Res<DroneResources>,
    mut ev_spawn_drone: EventReader<SpawnDroneEvent>,
) {
    for ev in ev_spawn_drone.iter() {
        commands
            .spawn(resources[ev.drone].clone())
            .insert(SpatialBundle::from_transform(ev.transform))
            .insert(aiming::GunLayer::default())
            .insert(RigidBody::Dynamic)
            .insert(Velocity::default())
            .insert(ExternalForce {
                force: Vec3::new(0.0, 0.0, 0.0),
                torque: Vec3::ZERO,
            })
            .insert(scene_setup::SetupRequired::new(
                move |commands, entities| {
                    let root = entities.iter().find(|e| e.contains::<SceneInstance>());

                    let collider_parts: Vec<_> = entities
                        .iter()
                        // Skip entities with `Handle<Mesh>` to operate only with GLTF's Nodes
                        .filter(|e| !e.contains::<Handle<Mesh>>())
                        .filter(
                            |e| matches!(e.get::<Name>(), Some(name) if name.starts_with("body")),
                        )
                        .map(|entity| entity.id())
                        .collect();

                    // Assign guns to entities named "barrel"
                    let guns: Vec<_> = entities
                        .iter()
                        // Skip entities with `Handle<Mesh>` to operate only with GLTF's Nodes
                        .filter(|e| !e.contains::<Handle<Mesh>>())
                        .filter(
                            |e| matches!(e.get::<Name>(), Some(name) if name.starts_with("barrel")),
                        )
                        .map(|e| {
                            commands.entity(e.id()).insert(weapon::MachineGun::new(5.0));
                            e.id()
                        })
                        .collect();

                    commands
                        .entity(root.unwrap().id())
                        .insert(collider_setup::ConvexHull::new(collider_parts))
                        .insert(Guns(guns));
                },
            ));
    }
}

fn orientation(mut drones: Query<(&aiming::GunLayer, &RotationSpeed, &mut Velocity)>) {
    for (gun_layer, rotation_speed, mut velocity) in drones.iter_mut() {
        velocity.angvel = (gun_layer.axis * gun_layer.angle).clamp_length_max(rotation_speed.0);
    }
}

fn fire_control(drones: Query<(&aiming::GunLayer, &Guns)>, mut triggers: Query<&mut gun::Trigger>) {
    for (gun_layer, guns) in drones.iter() {
        let threshold = if gun_layer.distance > 100.0 {
            // let's say for simplicity that target is 10m size
            10.0 / gun_layer.distance
        } else {
            0.3
        };
        if gun_layer.distance != 0.0 && gun_layer.angle < threshold {
            for gun in guns.0.iter() {
                if let Ok(mut gun_trigger) = triggers.get_mut(*gun) {
                    gun_trigger.pull();
                }
            }
        }
    }
}

pub struct DronePlugin;
impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_drone_resources)
            .add_event::<SpawnDroneEvent>()
            .add_system(spawn_drone)
            .add_system(orientation.after(aiming::gun_layer))
            .add_system(fire_control.after(orientation));
    }
}
