use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Annotates an entity to be used for building direction vector to the specified target.
#[derive(Component, Default)]
pub struct GunLayer {
    target: Option<Entity>,
    pub axis: Vec3,
    pub angle: f32,
    pub distance: f32,
}

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub enum Fraction {
    Drones,
    Turrets,
}

fn aiming_vector(origin: Vec3, mut target_pos: Vec3, velocity: Option<&Velocity>) -> Vec3 {
    // Adjust target position to compensate its velocity if any
    if let Some(velocity) = velocity {
        // todo: get from parameter
        let projectile_speed = 100.0;
        let time = target_pos.distance(origin) / projectile_speed;
        target_pos += velocity.linvel * time;
    }
    target_pos - origin
}

fn select_target(
    mut query: Query<(Option<&Fraction>, &GlobalTransform, &mut GunLayer)>,
    targets: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&Fraction>,
            Option<&Velocity>,
        ),
        (With<Collider>, Without<Sensor>),
    >,
) {
    for (own_fraction, transform, mut gun_layer) in query.iter_mut() {
        if !matches!(gun_layer.target, Some(target) if targets.contains(target)) {
            let forward_direction = transform.forward();
            let origin = transform.translation();

            gun_layer.target = targets
                .iter()
                .filter(|(_, _, fraction, _)| {
                    // Don't select targets with the same fraction
                    !matches!((own_fraction, fraction), (Some(lha), Some(rha)) if *lha == **rha)
                })
                .map(|(entity, transform, _, velocity)| {
                    let to_target = aiming_vector(origin, transform.translation(), velocity);
                    (entity, to_target, to_target.length_squared())
                })
                // todo: consider spatial optimizations to speed up lookup
                .filter(|(_, _, sqrared_distance)| {
                    // todo: Fix visibility distance once drones become smart enough not to fly away without a target
                    // const DEFAULT_VISIBILITY_SQARED_RANGE: f32 = 1000.0 * 1000.0;
                    0.0 < *sqrared_distance // && *sqrared_distance < DEFAULT_VISIBILITY_SQARED_RANGE
                })
                // find closest target to `forward_direction` to reduce required rotations
                // convert to integer with 2 digits precision to workaround that f32 is not Ord
                .max_by_key(|(_, to_target, sqrared_distance)| {
                    (to_target.dot(forward_direction) / sqrared_distance.sqrt() * 100.0) as i32
                })
                .map(|(entity, _, _)| entity);
        }
    }
}

pub fn gun_layer(
    mut query: Query<(&GlobalTransform, &mut GunLayer)>,
    targets: Query<(&GlobalTransform, Option<&Velocity>)>,
) {
    for (transform, mut gun_layer) in query.iter_mut() {
        let Some((target, velocity)) = gun_layer.target.and_then(|e| targets.get(e).ok()) else {
            // Target is not selected or not exists anymore - nothing to do.
            gun_layer.angle = 0.0;
            gun_layer.distance = 0.0;
            continue;
        };

        let to_target = aiming_vector(transform.translation(), target.translation(), velocity);
        let distance = to_target.length();
        let direction = to_target * distance.recip();

        gun_layer.distance = distance;
        // Required rotation to align gun layer orientation with `direction`
        (gun_layer.axis, gun_layer.angle) =
            Quat::from_rotation_arc(transform.forward(), direction).to_axis_angle();
    }
}

pub struct AimingPlugin;
impl Plugin for AimingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(select_target).add_system(gun_layer);
    }
}
