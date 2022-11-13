use bevy::{
    ecs::change_detection::DetectChanges, math::Affine3A, prelude::*,
    render::mesh::VertexAttributeValues,
};
use bevy_rapier3d::prelude::Collider;

/// Annotates an entity where a new collider should be added.
/// A new collider is computed as a convex hull that covers all meshes of `collider_parts` or theirs
/// direct children (no recursive traversal).
/// Children are traversed only if entity has no attached meshes, like GLTF Node.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SetupRequired(Vec<Entity>);

impl SetupRequired {
    pub fn new(collider_parts: Vec<Entity>) -> Self {
        Self(collider_parts)
    }
}

fn extract_mesh_vertices(mesh: &Mesh) -> Option<Vec<Vec3>> {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        VertexAttributeValues::Float32(vtx) => {
            Some(vtx.chunks(3).map(|v| Vec3::new(v[0], v[1], v[2])).collect())
        }
        VertexAttributeValues::Float32x3(vtx) => {
            Some(vtx.iter().map(|v| Vec3::new(v[0], v[1], v[2])).collect())
        }
        _ => None,
    }
}

fn setup_collider(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    to_setup: Query<(Entity, &SetupRequired, &GlobalTransform)>,
    with_children: Query<&Children>,
    with_meshes: Query<(&Handle<Mesh>, &GlobalTransform)>,
    mut with_transform: Query<&mut Transform, With<SetupRequired>>,
) {
    let extract_vertices = |mesh, affine: Affine3A| {
        // todo: consider Vec3 -> Vec3A and sort() + dedup() to speed up verices processing
        extract_mesh_vertices(meshes.get(mesh).unwrap())
            .unwrap_or(vec![])
            .into_iter()
            .map(move |v| affine.transform_point3(v))
    };

    for (entity, collider_parts, transform) in to_setup.iter() {
        // Collect all vertices in the world's transform
        let mut vertices = vec![];
        for part in collider_parts.0.iter() {
            // Try to get mesh from `part` entity
            if let Ok((mesh, transform)) = with_meshes.get(*part) {
                vertices.extend(extract_vertices(mesh, transform.affine()));
            } else {
                // Traverse `part` children and get meshes if any
                if let Ok(children) = with_children.get(*part) {
                    for child in children.iter() {
                        if let Ok((mesh, transform)) = with_meshes.get(*child) {
                            vertices.extend(extract_vertices(mesh, transform.affine()));
                        }
                    }
                }
            }
        }

        if !vertices.is_empty() {
            // With inverse transform, collider will match to the entity's shape
            let affine = transform.affine().inverse();
            vertices
                .iter_mut()
                .for_each(|v| *v = affine.transform_point3(*v));

            if let Some(collider) = Collider::convex_hull(&vertices) {
                commands.entity(entity).insert(collider);

                // Manual set `Changed` to the entity transform to trigger added collider position recalculation
                if let Ok(mut transform) = with_transform.get_mut(entity) {
                    transform.set_changed();
                }
            }
        }

        commands.entity(entity).remove::<SetupRequired>();
    }
}

pub struct ColliderSetupPlugin;
impl Plugin for ColliderSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_collider);
    }
}
