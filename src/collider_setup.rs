use bevy::{
    ecs::change_detection::DetectChanges,
    math::Affine3A,
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use bevy_rapier3d::prelude::{Collider, VHACDParameters};

/// Annotates an entity where a new collider should be added.
/// A new collider is computed as a convex hull that covers all meshes of `collider_parts` or theirs
/// direct children (no recursive traversal).
/// Children are traversed only if entity has no attached meshes, like GLTF Node.
#[derive(Component)]
pub struct ConvexHull(Vec<Entity>);

impl ConvexHull {
    pub fn new(collider_parts: Vec<Entity>) -> Self {
        Self(collider_parts)
    }
}

/// Annotates an entity where a new collider should be added.
/// A new collider is computed as a convex decomposition from mesh, taken from referenced entity.
/// This component use entity instead of Handle<Mesh> to resolve transform, applied to the mesh.
#[derive(Component)]
pub struct ConvexDecomposition {
    pub mesh_source: Entity,
    pub parameters: VHACDParameters,
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

fn extract_mesh_indices(mesh: &Mesh) -> Option<Vec<[u32; 3]>> {
    match mesh.indices() {
        Some(Indices::U16(idx)) => Some(
            idx.chunks_exact(3)
                .map(|i| [i[0] as u32, i[1] as u32, i[2] as u32])
                .collect(),
        ),
        Some(Indices::U32(idx)) => Some(idx.chunks_exact(3).map(|i| [i[0], i[1], i[2]]).collect()),
        None => None,
    }
}

fn convex_hull(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    to_setup: Query<(Entity, &ConvexHull, &GlobalTransform)>,
    with_children: Query<&Children>,
    with_meshes: Query<(&Handle<Mesh>, &GlobalTransform)>,
) {
    let extract_vertices = |mesh, affine: Affine3A| {
        // todo: consider Vec3 -> Vec3A and sort() + dedup() to speed up verices processing
        extract_mesh_vertices(meshes.get(mesh).unwrap())
            .unwrap_or_default()
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
                commands
                    .entity(entity)
                    .insert(collider)
                    .insert(RecalculateTransform);
            }
        }
        commands.entity(entity).remove::<ConvexHull>();
    }
}

fn convex_decomposition(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    to_setup: Query<(Entity, &ConvexDecomposition, &GlobalTransform)>,
    with_meshes: Query<(&Handle<Mesh>, &GlobalTransform)>,
) {
    for (entity, decomposition, transform) in to_setup.iter() {
        let (mesh, source_transform) = with_meshes.get(decomposition.mesh_source).unwrap();
        let mesh = meshes.get(mesh).unwrap();
        let mut vertices = extract_mesh_vertices(mesh).unwrap();
        let indices = extract_mesh_indices(mesh).unwrap();

        let to_global = source_transform.affine();
        let to_local = transform.affine().inverse();
        for v in vertices.iter_mut() {
            *v = to_local.transform_point3(to_global.transform_point3(*v));
        }

        commands
            .entity(entity)
            .insert(Collider::convex_decomposition_with_params(
                &vertices,
                &indices,
                &decomposition.parameters,
            ))
            .insert(RecalculateTransform);
        commands.entity(entity).remove::<ConvexDecomposition>();
    }
}

/// Add this component if manual transform recalculation triggering is required.
/// A common case for this is adding collider to the stationary entity, as collider will be spawned at [0.0, 0.0, 0.0],
/// and will be moved to the correct possition only once GlobalTransform is recalculated .
#[derive(Component)]
struct RecalculateTransform;

fn recalculate_transform(
    mut commands: Commands,
    mut transforms: Query<(Entity, &mut Transform), With<RecalculateTransform>>,
) {
    for (entity, mut transform) in transforms.iter_mut() {
        transform.set_changed();
        commands.entity(entity).remove::<RecalculateTransform>();
    }
}

pub struct ColliderSetupPlugin;
impl Plugin for ColliderSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(convex_hull)
            .add_system(convex_decomposition)
            .add_system(recalculate_transform);
    }
}
