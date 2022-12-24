/// Inspired by https://github.com/nicopap/bevy-scene-hook
use bevy::{asset::LoadState, ecs::world::EntityRef, prelude::*, scene::SceneInstance};

/// Component to attach setup function that will be invoked once scene is loaded.
/// Provided callback will receive GLTF Nodes.
///
/// Example:
///
/// ```
/// commands
///     .spawn(SceneBundle {
///         scene: asset_server.load("my_scene.glb#Scene0"),
///         ..default()
///     })
///     .insert(SetupRequired::new(|commands, nodes| {
///         nodes
///             .iter()
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
pub struct SetupRequired(Box<dyn Fn(&mut Commands, &[EntityRef]) + Send + Sync + 'static>);

impl SetupRequired {
    pub fn new<F: Fn(&mut Commands, &[EntityRef]) + Send + Sync + 'static>(setup_fn: F) -> Self {
        Self(Box::new(setup_fn))
    }
}

fn setup_scene(
    scenes: Query<(Entity, &Handle<Scene>, &SceneInstance, &SetupRequired)>,
    server: Res<AssetServer>,
    scene_manager: Res<SceneSpawner>,
    world: &World,
    mut commands: Commands,
) {
    for (entity, handle, instance, setup) in scenes.iter() {
        if server.get_load_state(handle.id()) == LoadState::Loaded {
            let instance_entities = scene_manager.iter_instance_entities(**instance);
            let entities: Vec<_> = std::iter::once(entity) // add the root entity to make possible to modify once scene is loaded
                .chain(instance_entities)
                .filter_map(|e| world.get_entity(e))
                // Skip entities with `Handle<Mesh>` to operate only with GLTF's Nodes
                .filter(|e| !e.contains::<Handle<Mesh>>())
                // storing result of filtering allows us to handle lifetime problems and
                // workaround `Box<dyn Iterator<Item = EntityRef>>` in function type declaration
                .collect();
            setup.0(&mut commands, &entities);
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
