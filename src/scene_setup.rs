/// Inspired by https://github.com/nicopap/bevy-scene-hook
use bevy::{ecs::world::EntityRef, prelude::*, scene::SceneInstance};

/// Component to attach setup function that will be invoked once scene is loaded.
///
/// Example:
///
/// ```
/// commands
///     .spawn_bundle(SceneBundle {
///         scene: asset_server.load("my_scene.glb#Scene0"),
///         ..default()
///     })
///     .insert(SetupRequired::new(|commands, entities| {
///         entities
///             .filter(|e| e.get::<Handle<Mesh>>().is_none()) // Skip GLTF Mesh entities
///             .filter_map(|e| e.get::<Name>().map(|name| (e.id(), name)))
///             .for_each(|(entity, name)| {
///                 if name.starts_with("Muzzle") {
///                     commands.entity(entity).insert(Muzzle);
///                 } else if name.starts_with("Body") {
///                     commands.entity(entity).insert(Body);
///                 } else if name.starts_with("Head") {
///                     commands.entity(entity).insert(Head);
///                 }
///             });
///     }));
/// ```
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SetupRequired(
    Box<dyn Fn(&mut Commands, std::slice::Iter<EntityRef>) + Send + Sync + 'static>,
);

impl SetupRequired {
    pub fn new<F: Fn(&mut Commands, std::slice::Iter<EntityRef>) + Send + Sync + 'static>(
        setup_fn: F,
    ) -> Self {
        Self(Box::new(setup_fn))
    }
}

fn setup_scene(
    scenes: Query<(Entity, &SceneInstance, &SetupRequired)>,
    scene_manager: Res<SceneSpawner>,
    world: &World,
    mut commands: Commands,
) {
    for (entity, instance, setup) in scenes.iter() {
        if let Some(entities) = scene_manager.iter_instance_entities(**instance) {
            setup.0(
                &mut commands,
                [entity] // add the root entity to make possible to modify once scene is loaded
                    .into_iter()
                    .chain(entities)
                    .filter_map(|e| world.get_entity(e))
                    // collect() + iter() allows to handle lifetime problems and
                    // workarounds `Box<dyn Iterator<Item = EntityRef>>` in function type declaration
                    .collect::<Vec<_>>()
                    .iter(),
            );
            commands.entity(entity).remove::<SetupRequired>();
        }
    }
}

pub struct SceneSetupPlugin;
impl Plugin for SceneSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_scene);
    }
}
