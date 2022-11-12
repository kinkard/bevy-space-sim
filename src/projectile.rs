use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::*;
use bevy_hanabi::*;
use bevy_rapier3d::prelude::*;

/// Entity lifetime in seconds, after which entity should be destroyed
#[derive(Component, Clone)]
pub struct Lifetime(pub f32);

fn lifetime(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime)>) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.0 -= time.delta_seconds();
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component, Clone)]
pub struct Damage(pub u32);

#[derive(Component)]
pub struct HitPoints {
    maximum: u32,
    current: u32,
}

impl HitPoints {
    pub fn new(maximum: u32) -> Self {
        HitPoints {
            maximum,
            current: maximum,
        }
    }
    pub fn percent(&self) -> u32 {
        100 * self.current / self.maximum
    }
    pub fn dead(&self) -> bool {
        self.current == 0
    }
    pub fn hit(&mut self, damage: u32) -> &mut Self {
        self.current = self.current.saturating_sub(damage);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::HitPoints;

    #[test]
    fn test_new_hp_always_100() {
        assert_eq!(HitPoints::new(1).percent(), 100);
        assert_eq!(HitPoints::new(2).percent(), 100);
        assert_eq!(HitPoints::new(111).percent(), 100);
    }

    #[test]
    fn test_hp_hit() {
        assert_eq!(HitPoints::new(1).hit(0).percent(), 100);
        assert_eq!(HitPoints::new(1).hit(1).percent(), 0);
        assert_eq!(HitPoints::new(1).hit(10).percent(), 0);
        assert_eq!(HitPoints::new(50).hit(25).percent(), 50);
        assert_eq!(HitPoints::new(100).hit(0).percent(), 100);
        assert_eq!(HitPoints::new(100).hit(1).percent(), 99);
        assert_eq!(HitPoints::new(100).hit(99).percent(), 1);
        assert_eq!(HitPoints::new(100).hit(100).percent(), 0);
        assert_eq!(HitPoints::new(100).hit(101).percent(), 0);

        assert!(!HitPoints::new(1).hit(0).dead());
        assert!(HitPoints::new(1).hit(1).dead());
        assert!(HitPoints::new(1).hit(10).dead());
        assert!(!HitPoints::new(50).hit(25).dead());
        assert!(!HitPoints::new(100).hit(0).dead());
        assert!(!HitPoints::new(100).hit(1).dead());
        assert!(!HitPoints::new(100).hit(99).dead());
        assert!(HitPoints::new(100).hit(100).dead());
        assert!(HitPoints::new(100).hit(101).dead());
    }
}

/// Entity explosion effect. If set - entity will be destroyed on collision
/// with spawning a corresponding effect.
#[derive(Component, Copy, Clone, PartialEq)]
pub enum ExplosionEffect {
    Debug,
    Small,
    Big,
}

impl Default for ExplosionEffect {
    fn default() -> Self {
        ExplosionEffect::Debug
    }
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    #[bundle]
    pub mesh_material: PbrBundle,
    pub collider: Collider,
    pub velocity: Velocity,
    pub lifetime: Lifetime,
    pub explosion: ExplosionEffect,
    pub damage: Damage,
    pub events: ActiveEvents,
    pub rigid_body: RigidBody,
    pub sensor: Sensor,
    // todo: would be nice to measure it's impact on performance
    pub no_shadow_caster: NotShadowCaster,
    pub no_shadow_receiver: NotShadowReceiver,
    pub name: Name,
}

impl Default for ProjectileBundle {
    fn default() -> Self {
        Self {
            mesh_material: PbrBundle::default(),
            collider: Collider::default(),
            velocity: Velocity::default(),
            lifetime: Lifetime(10.0),
            explosion: ExplosionEffect::default(),
            damage: Damage(0),
            events: ActiveEvents::COLLISION_EVENTS,
            rigid_body: RigidBody::Dynamic,
            sensor: Sensor,
            no_shadow_caster: NotShadowCaster,
            no_shadow_receiver: NotShadowReceiver,
            name: Name::new("Projectile"),
        }
    }
}

fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // Create a default explosion effect
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Color::PINK.into());
    color_gradient.add_key(0.4, Color::PINK.into());
    color_gradient.add_key(1.0, Color::NONE.into());

    commands
        .spawn_bundle(ParticleEffectBundle::new(
            effects.add(
                EffectAsset {
                    capacity: 1024,
                    spawner: Spawner::once(64.0.into(), false),
                    ..default()
                }
                .init(PositionSphereModifier {
                    radius: 0.1,
                    speed: 0.5.into(),
                    dimension: ShapeDimension::Surface,
                    ..default()
                })
                .init(ParticleLifetimeModifier { lifetime: 10.0 })
                // .render(ParticleTextureModifier {
                //     texture: asset_server.load("textures/cloud.png"),
                // })
                .render(BillboardModifier)
                .render(SizeOverLifetimeModifier {
                    gradient: Gradient::constant(Vec2::splat(0.1)),
                })
                .render(ColorOverLifetimeModifier {
                    gradient: color_gradient,
                }),
            ),
        ))
        .insert(ExplosionEffect::Debug)
        .insert(Name::new("ExplosionEffect::Debug"));

    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Color::WHITE.into());
    color_gradient.add_key(0.1, Color::YELLOW.into());
    color_gradient.add_key(0.4, Color::RED.into());
    color_gradient.add_key(1.0, Color::NONE.into());

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec2::splat(0.05));
    size_gradient.add_key(1.0, Vec2::splat(0.2));

    commands
        .spawn_bundle(ParticleEffectBundle::new(
            effects.add(
                EffectAsset {
                    capacity: 16384,
                    spawner: Spawner::once(1024.0.into(), false),
                    ..default()
                }
                .init(PositionSphereModifier {
                    radius: 0.2,
                    speed: 5.0.into(),
                    dimension: ShapeDimension::Surface,
                    ..default()
                })
                .init(ParticleLifetimeModifier { lifetime: 2.0 })
                // .render(ParticleTextureModifier {
                //     texture: asset_server.load("textures/cloud.png"),
                // })
                .render(BillboardModifier)
                .render(SizeOverLifetimeModifier {
                    gradient: size_gradient,
                })
                .render(ColorOverLifetimeModifier {
                    gradient: color_gradient,
                }),
            ),
        ))
        .insert(ExplosionEffect::Big)
        .insert(Name::new("ExplosionEffect::Big"));

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Color::WHITE.into());
    gradient.add_key(0.1, Color::YELLOW.into());
    gradient.add_key(0.4, Color::BLUE.into());
    gradient.add_key(1.0, Color::NONE.into());

    commands
        .spawn_bundle(ParticleEffectBundle::new(
            effects.add(
                EffectAsset {
                    capacity: 16384,
                    spawner: Spawner::once(128.0.into(), false),
                    ..default()
                }
                .init(PositionSphereModifier {
                    radius: 0.1,
                    speed: 5.0.into(),
                    dimension: ShapeDimension::Surface,
                    ..default()
                })
                .init(ParticleLifetimeModifier { lifetime: 0.3 })
                // .render(ParticleTextureModifier {
                //     texture: asset_server.load("textures/cloud.png"),
                // })
                .render(BillboardModifier)
                .render(SizeOverLifetimeModifier {
                    gradient: Gradient::constant(Vec2::splat(0.05)),
                })
                .render(ColorOverLifetimeModifier { gradient }),
            ),
        ))
        .insert(ExplosionEffect::Small)
        .insert(Name::new("ExplosionEffect::Small"));
}

fn hit_collision(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    projectiles: Query<&Damage>,
    mut targets: Query<&mut HitPoints>,
) {
    for event in collisions.iter() {
        if let CollisionEvent::Started(first, second, _) = event {
            for (projectile, target) in [(first, second), (second, first)] {
                if let (Ok(damage), Ok(mut hp)) =
                    (projectiles.get(*projectile), targets.get_mut(*target))
                {
                    if hp.hit(damage.0).dead() {
                        commands.entity(*target).despawn_recursive();
                    }
                }
            }
        }
    }
}

fn explosive_collision(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    mut explosions: Query<(&ExplosionEffect, &mut ParticleEffect, &mut Transform)>,
    explosives: Query<(&ExplosionEffect, &Transform), Without<ParticleEffect>>,
) {
    for event in collisions.iter() {
        if let CollisionEvent::Started(first, second, _) = event {
            for entity in [first, second] {
                // If collided entity is explosive
                if let Ok((&explosive, transform)) = explosives.get(*entity) {
                    // Match effect by it's type or use `Debug` if can't find
                    let mut explosion = explosions
                        .iter_mut()
                        .find(|(&effect, _, _)| effect == explosive);
                    if explosion.is_none() {
                        explosion = explosions
                            .iter_mut()
                            .find(|(&effect, _, _)| effect == ExplosionEffect::Debug);
                    }

                    let (_, mut effect, mut effect_transform) = explosion.unwrap();
                    effect_transform.translation = transform.translation;
                    effect.maybe_spawner().unwrap().reset();

                    // destroy every explosive entity on collision
                    commands.entity(*entity).despawn_recursive();
                }
            }
        }
    }
}

pub struct ProjectilePlugin;
impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(HanabiPlugin)
            .add_startup_system(setup)
            .add_system(lifetime)
            .add_system(hit_collision)
            .add_system(explosive_collision);
    }
}
