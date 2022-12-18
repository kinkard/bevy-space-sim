use bevy::{prelude::*, scene::SceneInstance};
use bevy_rapier3d::prelude::*;
use std::ops::{Index, IndexMut};

use crate::{collider_setup, projectile, scene_setup, weapon};

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

#[derive(Default)]
struct DroneCfg {
    scene: Handle<Scene>,
    name: Name,
    hitpoints: projectile::HitPoints,
}

#[derive(Resource, Default)]
struct DroneResources([DroneCfg; 2]);

impl Index<Drone> for DroneResources {
    type Output = DroneCfg;
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
    resources[Drone::Praetor] = DroneCfg {
        scene: assets.load("models/praetor.glb#Scene0"),
        name: Name::new("Drone::Praetor"),
        hitpoints: projectile::HitPoints::new(300),
    };
    resources[Drone::Infiltrator] = DroneCfg {
        scene: assets.load("models/infiltrator.glb#Scene0"),
        name: Name::new("Drone::Infiltrator"),
        hitpoints: projectile::HitPoints::new(200),
    };
    commands.insert_resource(resources);
}

fn spawn_drone(
    mut commands: Commands,
    resources: Res<DroneResources>,
    mut ev_spawn_drone: EventReader<SpawnDroneEvent>,
) {
    for ev in ev_spawn_drone.iter() {
        let drone_cfg = &resources[ev.drone];

        commands
            .spawn(SceneBundle {
                scene: drone_cfg.scene.clone(),
                transform: ev.transform,
                ..default()
            })
            .insert(drone_cfg.hitpoints.clone())
            .insert(drone_cfg.name.clone())
            .insert(RigidBody::Dynamic)
            .insert(ExternalForce {
                force: Vec3::new(0.0, 0.0, 0.0),
                torque: Vec3::ZERO,
            })
            .insert(Velocity::default())
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
                    commands
                        .entity(root.unwrap().id())
                        .insert(collider_setup::ConvexHull::new(collider_parts));

                    // Assign guns to entities named "barrel"
                    entities
                        .iter()
                        // Skip entities with `Handle<Mesh>` to operate only with GLTF's Nodes
                        .filter(|e| !e.contains::<Handle<Mesh>>())
                        .filter(
                            |e| matches!(e.get::<Name>(), Some(name) if name.starts_with("barrel")),
                        )
                        .for_each(|e| {
                            commands.entity(e.id()).insert(weapon::MachineGun::new(5.0));
                        });
                },
            ));
    }
}

pub struct DronePlugin;
impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_drone_resources)
            .add_event::<SpawnDroneEvent>()
            .add_system(spawn_drone);
    }
}
