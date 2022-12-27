use bevy::prelude::*;

use crate::gun;

#[derive(Bundle)]
pub struct FlakCannon {
    trigger: gun::Trigger,
    gun: gun::Gun,
    barrels: gun::MultiBarrel,
}

impl FlakCannon {
    /// Each entity in the `barrels` should have `Barrel` component
    pub fn new(barrels: Vec<Entity>, rate_of_fire: f32) -> Self {
        Self {
            trigger: gun::Trigger::default(),
            gun: gun::Gun::new(rate_of_fire, gun::Projectile::Bullet, 200.0),
            barrels: gun::MultiBarrel::new(barrels),
        }
    }
}

#[derive(Bundle)]
pub struct MachineGun {
    trigger: gun::Trigger,
    gun: gun::Gun,
}

impl MachineGun {
    pub fn new(rate_of_fire: f32) -> Self {
        Self {
            trigger: gun::Trigger::default(),
            gun: gun::Gun::new(rate_of_fire, gun::Projectile::Bullet, 200.0),
        }
    }
}

#[derive(Bundle)]
pub struct RocketLauncher {
    trigger: gun::Trigger,
    gun: gun::Gun,
}

impl RocketLauncher {
    pub fn new(rate_of_fire: f32) -> Self {
        Self {
            trigger: gun::Trigger::default(),
            gun: gun::Gun::new(rate_of_fire, gun::Projectile::Rocket, 20.0),
        }
    }
}
