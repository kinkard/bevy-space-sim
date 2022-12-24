use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::projectile;

#[derive(Component, Default)]
pub struct Trigger {
    is_pulled: bool,
}

impl Trigger {
    pub fn pull(&mut self) {
        self.is_pulled = true;
    }
}

pub enum Projectile {
    Bullet,
    Rocket,
}

#[derive(Component)]
pub struct Gun {
    rate_of_fire_timer: Timer,
    projectile: Projectile,
    speed: f32,
}

impl Gun {
    pub fn new(rate_of_fire: f32, projectile: Projectile, speed: f32) -> Self {
        Self {
            rate_of_fire_timer: Timer::from_seconds(1.0 / rate_of_fire, TimerMode::Repeating),
            projectile,
            speed,
        }
    }
}

fn check_trigger(mut guns: Query<(&mut Trigger, &mut Gun)>, time: Res<Time>) {
    for (mut trigger, mut gun) in guns.iter_mut() {
        gun.rate_of_fire_timer.tick(time.delta());

        if trigger.is_pulled {
            trigger.is_pulled = false;

            if gun.rate_of_fire_timer.paused() {
                gun.rate_of_fire_timer.unpause();
                let duration = gun.rate_of_fire_timer.duration();
                gun.rate_of_fire_timer.tick(duration);
            }
        } else if gun.rate_of_fire_timer.just_finished() {
            gun.rate_of_fire_timer.reset();
            gun.rate_of_fire_timer.pause();
        }
    }
}

/// Annotates entities that are used as projectile spawn bullets for FlakCannon
#[derive(Component)]
pub struct Barrel;

/// Link to the entities with `Barrel` component, used to spawn bullets
#[derive(Component)]
pub struct MultiBarrel(Vec<Entity>);

impl MultiBarrel {
    pub fn new(barrels: Vec<Entity>) -> Self {
        Self(barrels)
    }
}

#[derive(Resource)]
struct Bullet {
    collider: Collider,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,

    lifetime: projectile::Lifetime,

    explosion: projectile::ExplosionEffect,
    damage: projectile::Damage,
}

impl Bullet {
    fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let radius = 0.02;
        Self {
            collider: Collider::capsule_y(8.0 * radius, radius),
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
            lifetime: projectile::Lifetime(10.0),
            explosion: projectile::ExplosionEffect::Small,
            damage: projectile::Damage(1),
        }
    }

    fn spawn(&self, commands: &mut Commands, position: Vec3, direction: Vec3, speed: f32) {
        commands.spawn(projectile::ProjectileBundle {
            mesh_material: PbrBundle {
                mesh: self.mesh.clone(),
                material: self.material.clone(),
                transform: Transform {
                    translation: position,
                    // `Collider::capsule_y` and `shape::Capsule` are both aligned with Vec3::Y axis
                    rotation: Quat::from_rotation_arc(Vec3::Y, direction),
                    scale: Vec3::ONE,
                },
                ..default()
            },
            collider: self.collider.clone(),
            velocity: Velocity {
                linvel: direction * speed,
                ..default()
            },
            lifetime: self.lifetime.clone(),
            explosion: self.explosion,
            damage: self.damage.clone(),
            ..default()
        });
    }
}

#[derive(Resource)]
struct Rocket {
    collider: Collider,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,

    lifetime: projectile::Lifetime,

    explosion: projectile::ExplosionEffect,
    damage: projectile::Damage,

    light: PointLight,
}

impl Rocket {
    fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let radius = 0.2;
        Self {
            collider: Collider::ball(radius),
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
            lifetime: projectile::Lifetime(30.0),
            explosion: projectile::ExplosionEffect::Big,
            damage: projectile::Damage(99),
            light: PointLight {
                intensity: 1500.0,
                radius,
                color: Color::rgb(1.0, 0.2, 0.2),
                ..default()
            },
        }
    }

    fn spawn(&self, commands: &mut Commands, position: Vec3, direction: Vec3, speed: f32) {
        commands
            .spawn(projectile::ProjectileBundle {
                mesh_material: PbrBundle {
                    mesh: self.mesh.clone(),
                    material: self.material.clone(),
                    transform: Transform {
                        translation: position,
                        // `Collider::capsule_y` and `shape::Capsule` are both aligned with Vec3::Y axis
                        rotation: Quat::from_rotation_arc(Vec3::Y, direction),
                        scale: Vec3::ONE,
                    },
                    ..default()
                },
                collider: self.collider.clone(),
                velocity: Velocity {
                    linvel: direction * speed,
                    ..default()
                },
                lifetime: self.lifetime.clone(),
                explosion: self.explosion,
                damage: self.damage.clone(),
                ..default()
            })
            .with_children(|children| {
                children.spawn(PointLightBundle {
                    point_light: self.light.clone(),
                    ..default()
                });
            });
    }
}

fn setup_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Bullet::new(&mut meshes, &mut materials));
    commands.insert_resource(Rocket::new(&mut meshes, &mut materials));
}

fn single_barrel(
    mut commands: Commands,
    guns: Query<(&GlobalTransform, &Gun), Without<MultiBarrel>>,
    bullet: Res<Bullet>,
    rocket: Res<Rocket>,
) {
    for (barrel, gun) in guns.iter() {
        if gun.rate_of_fire_timer.just_finished() {
            // todo: move this code somewhere and make it possible to add more different projectiles
            match gun.projectile {
                Projectile::Bullet => bullet.spawn(
                    &mut commands,
                    barrel.translation(),
                    barrel.forward(),
                    gun.speed,
                ),
                Projectile::Rocket => rocket.spawn(
                    &mut commands,
                    barrel.translation(),
                    barrel.forward(),
                    gun.speed,
                ),
            };
        }
    }
}

fn multi_barrel(
    mut commands: Commands,
    guns: Query<(&Gun, &MultiBarrel)>,
    barrel_transforms: Query<&GlobalTransform, With<Barrel>>,
    projectile: Res<Bullet>,
) {
    for (gun, barrels) in guns.iter() {
        if gun.rate_of_fire_timer.just_finished() {
            for barrel in barrels.0.iter() {
                let barrel = barrel_transforms.get(*barrel).unwrap();
                projectile.spawn(
                    &mut commands,
                    barrel.translation(),
                    barrel.forward(),
                    gun.speed,
                );
            }
        }
    }
}

pub struct GunPlugin;
impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_projectile)
            .add_system(check_trigger)
            .add_system(single_barrel)
            .add_system(multi_barrel);
    }
}
