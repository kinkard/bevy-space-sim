// Based on https://github.com/bevyengine/bevy/blob/main/examples/3d/skybox.rs
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, AsBindGroupError, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            OwnedBindingResource, PreparedBindGroup, RenderPipelineDescriptor, SamplerBindingType,
            ShaderRef, ShaderStages, SpecializedMeshPipelineError, TextureSampleType,
            TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::{CompressedImageFormats, FallbackImage},
    },
};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "9509a0f8-3c05-48ee-a13e-a93226c7f488"]
struct CubemapMaterial {
    texture: Option<Handle<Image>>,
}

impl Material for CubemapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cubemap_unlit.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

impl AsBindGroup for CubemapMaterial {
    type Data = ();

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        _fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<Self>, AsBindGroupError> {
        let texture = self
            .texture
            .as_ref()
            .ok_or(AsBindGroupError::RetryNextUpdate)?;
        let image = images
            .get(texture)
            .ok_or(AsBindGroupError::RetryNextUpdate)?;
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&image.sampler),
                },
            ],
            label: Some("cubemap_texture_material_bind_group"),
            layout,
        });

        Ok(PreparedBindGroup {
            bind_group,
            bindings: vec![
                OwnedBindingResource::TextureView(image.texture_view.clone()),
                OwnedBindingResource::Sampler(image.sampler.clone()),
            ],
            data: (),
        })
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // Cubemap Base Color Texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                    },
                    count: None,
                },
                // Cubemap Base Color Texture Sampler
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        })
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cubemap_materials: ResMut<Assets<CubemapMaterial>>,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {
    // Cubemap is generated by https://github.com/petrocket/spacescape, http://alexcpeterson.com/spacescape/
    // And encoded to ktx2 with ASTC encoding and zstd compression using https://github.com/KhronosGroup/KTX-Software:
    // `toktx --encode astc --astc_blk_d 4x4 --zcmp 19 --cubemap background posx.png negx.png posy.png negy.png posz.png negz.png`
    // Comparing to the simple PNG this saves 50Mb of RAM usage during runtime.
    assert!(
        CompressedImageFormats::from_features(render_device.features())
            .contains(CompressedImageFormats::ASTC_LDR)
    );
    let skybox_image = asset_server.load("textures/background_astc.ktx2");

    // Raw PNG also can be used with conversion to the cubemap using ImageMagick (see Unity coordinate system):
    // `convert posx.png negx.png posy.png negy.png posz.png negz.png -gravity center -append cubemap.png`
    // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
    // so they appear as one texture. The following code reconfigures the texture as necessary:
    // ```
    // let mut image = images.get_mut(&image_handle).unwrap();
    // if image.texture_descriptor.array_layer_count() == 1 {
    //     image.reinterpret_stacked_2d_as_array(
    //         image.texture_descriptor.size.height / image.texture_descriptor.size.width,
    //     );
    //     image.texture_view_descriptor = Some(TextureViewDescriptor {
    //         dimension: Some(TextureViewDimension::Cube),
    //         ..default()
    //     });
    // }
    // ```

    // TODO: consider setting skybox as a child to the camera
    commands
        .spawn(MaterialMeshBundle::<CubemapMaterial> {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 10000.0 })),
            material: cubemap_materials.add(CubemapMaterial {
                texture: skybox_image.into(),
            }),
            ..default()
        })
        .insert(Name::new("Skybox"));

    // Setup ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate colour and brightness to match
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(210, 220, 240),
        brightness: 0.1,
    });
}

pub struct SkyboxPlugin;
impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<CubemapMaterial>::default())
            .add_startup_system(setup);
    }
}
