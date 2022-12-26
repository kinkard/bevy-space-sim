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

fn aiming_vector(origin: Vec3, target_pos: Vec3, relative_vel: Vec3) -> Vec3 {
    // todo: get from parameter
    let projectile_speed = 100.0;

    let to_target = target_pos - origin;

    // solve quadratic equation around interception time
    // with known distance, target's velocity, projectile's velocity
    let squared_speed_diff = projectile_speed * projectile_speed - relative_vel.length_squared();
    let squared_distance = to_target.length_squared();
    let b = to_target.dot(relative_vel);
    let discriminant = b * b + squared_speed_diff * squared_distance;

    // if we found quadratic equation root(s) - use it, otherwise take zero
    // as zero is safe - no prediction is made
    let time = if discriminant >= 0.0 {
        let sqrt = discriminant.sqrt();
        let first_root = (b + sqrt) / squared_speed_diff;
        let second_root = (b - sqrt) / squared_speed_diff;
        if first_root > 0.0 && second_root > 0.0 {
            // if both times are valid - take the smallest one
            first_root.min(second_root)
        } else if first_root > 0.0 {
            first_root
        } else if second_root > 0.0 {
            second_root
        } else {
            0.0
        }
    } else {
        0.0
    };

    to_target + relative_vel * time
}

fn select_target(
    mut query: Query<(
        &GlobalTransform,
        Option<&Velocity>,
        Option<&Fraction>,
        &mut GunLayer,
    )>,
    targets: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&Velocity>,
            Option<&Fraction>,
        ),
        (With<Collider>, Without<Sensor>),
    >,
) {
    for (transform, own_velocity, own_fraction, mut gun_layer) in query.iter_mut() {
        if !matches!(gun_layer.target, Some(target) if targets.contains(target)) {
            let forward_direction = transform.forward();
            let origin = transform.translation();
            let own_vel = own_velocity.map(|v| v.linvel).unwrap_or_default();

            gun_layer.target = targets
                .iter()
                .filter(|(_, _, _, target_fraction)| {
                    // Don't select targets with the same fraction
                    !matches!((own_fraction, target_fraction), (Some(&own), Some(&target)) if own == target)
                })
                .map(|(entity, transform, velocity, _)| {
                    let target_vel = velocity.map(|v| v.linvel).unwrap_or_default();
                    let to_target =
                        aiming_vector(origin, transform.translation(), target_vel - own_vel);
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
    mut query: Query<(&GlobalTransform, Option<&Velocity>, &mut GunLayer)>,
    targets: Query<(&GlobalTransform, Option<&Velocity>)>,
) {
    for (transform, own_velocity, mut gun_layer) in query.iter_mut() {
        let Some((target, target_velocity)) = gun_layer.target.and_then(|e| targets.get(e).ok()) else {
            // Target is not selected or not exists anymore - nothing to do.
            gun_layer.angle = 0.0;
            gun_layer.distance = 0.0;
            continue;
        };

        let own_vel = own_velocity.map(|v| v.linvel).unwrap_or_default();
        let target_vel = target_velocity.map(|v| v.linvel).unwrap_or_default();

        let to_target = aiming_vector(
            transform.translation(),
            target.translation(),
            target_vel - own_vel,
        );
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
