use bevy::prelude::*;
use bevy_hanabi::*;
use bevy_rapier3d::prelude::*;

/// Entity lifetime in seconds, after which entity should be destroyed
#[derive(Component)]
pub struct Lifetime(pub f32);

fn lifetime(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime)>) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.0 -= time.delta_seconds();
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
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
    pub rigid_body: RigidBody,
    pub events: ActiveEvents,
    pub sensor: Sensor,
}

impl Default for ProjectileBundle {
    fn default() -> Self {
        Self {
            mesh_material: PbrBundle::default(),
            collider: Collider::default(),
            velocity: Velocity::default(),
            lifetime: Lifetime(10.0),
            explosion: ExplosionEffect::default(),
            rigid_body: RigidBody::Dynamic,
            events: ActiveEvents::COLLISION_EVENTS,
            sensor: Sensor,
        }
    }
}

fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // Create a default explosion effect
    commands
        .spawn_bundle(ParticleEffectBundle::new(
            effects.add(
                EffectAsset {
                    name: String::from("Default explosion"),
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
                    // PINK color
                    gradient: Gradient::constant(Color::PINK.into()),
                }),
            ),
        ))
        .insert(ExplosionEffect::Debug);

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
        .insert(ExplosionEffect::Big);

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
                    spawner: Spawner::once(64.0.into(), false),
                    ..default()
                }
                .init(PositionSphereModifier {
                    radius: 0.1,
                    speed: 5.0.into(),
                    dimension: ShapeDimension::Surface,
                    ..default()
                })
                .init(ParticleLifetimeModifier { lifetime: 0.2 })
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
        .insert(ExplosionEffect::Small);
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
            .add_system(explosive_collision);
    }
}
