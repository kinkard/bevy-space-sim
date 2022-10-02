use bevy::prelude::*;

/// Entity lifetime in seconds, after which entity will be destroyed
#[derive(Component)]
pub struct Lifetime(pub f32);

pub fn lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.0 -= time.delta_seconds();
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
