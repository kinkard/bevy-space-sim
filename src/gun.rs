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

#[derive(Component)]
pub struct Gun {
    rate_of_fire_timer: Timer,
}

impl Gun {
    pub fn new(rate_of_fire: f32) -> Self {
        Self {
            rate_of_fire_timer: Timer::from_seconds(1.0 / rate_of_fire, TimerMode::Repeating),
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
struct GunProjectile {
    collider: Collider,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,

    speed: f32,
    lifetime: projectile::Lifetime,

    explosion: projectile::ExplosionEffect,
    damage: projectile::Damage,
}

impl GunProjectile {
    fn new(
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        radius: f32,
        speed: f32,
        lifetime: projectile::Lifetime,
        explosion: projectile::ExplosionEffect,
        damage: projectile::Damage,
    ) -> Self {
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
            speed,
            damage,
            lifetime,
            explosion,
        }
    }

    fn spawn(&self, commands: &mut Commands, position: Vec3, direction: Vec3) {
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
                linvel: direction * self.speed,
                ..default()
            },
            lifetime: self.lifetime.clone(),
            explosion: self.explosion,
            damage: self.damage.clone(),
            ..default()
        });
    }
}

fn setup_projectile(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GunProjectile::new(
        meshes,
        materials,
        0.02,
        100.0,
        projectile::Lifetime(10.0),
        projectile::ExplosionEffect::Small,
        projectile::Damage(1),
    ));
}

fn single_barrel(
    mut commands: Commands,
    guns: Query<(&GlobalTransform, &Gun), Without<MultiBarrel>>,
    projectile: Res<GunProjectile>,
) {
    for (barrel, gun) in guns.iter() {
        if gun.rate_of_fire_timer.just_finished() {
            projectile.spawn(&mut commands, barrel.translation(), barrel.forward());
        }
    }
}

fn multi_barrel(
    mut commands: Commands,
    guns: Query<(&Gun, &MultiBarrel)>,
    barrel_transforms: Query<&GlobalTransform, With<Barrel>>,
    projectile: Res<GunProjectile>,
) {
    for (gun, barrels) in guns.iter() {
        if gun.rate_of_fire_timer.just_finished() {
            for barrel in barrels.0.iter() {
                let barrel = barrel_transforms.get(*barrel).unwrap();
                projectile.spawn(&mut commands, barrel.translation(), barrel.forward());
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
