//! Loads and renders a glTF file as a scene.

use std::f32::consts::*;
use bevy_fly_camera::{FlyCamera,FlyCameraPlugin};

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    render::camera::RenderTarget,
    window::WindowRef,
};


use bevy::{
    asset::LoadState,
    input::mouse::MouseMotion,
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
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::{CompressedImageFormats, FallbackImage},
    },
};

const CUBEMAP_SWAP_DELAY: f32 = 3.0;
const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[
    (
        "textures/space_cubemap.png",
        CompressedImageFormats::NONE,
    ),
    // (
    //     "textures/Ryfjallet_cubemap_astc4x4.ktx2",
    //     CompressedImageFormats::ASTC_LDR,
    // ),
    // (
        // "textures/Ryfjallet_cubemap_bc7.ktx2",
        // CompressedImageFormats::BC,
    // ),
    // (
    //     "textures/Ryfjallet_cubemap_etc2.ktx2",
    //     CompressedImageFormats::ETC2,
    // ),
];



#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup)
        .add_system(cycle_cubemap_asset)
        .add_system(asset_loaded.after(cycle_cubemap_asset))
        .add_system(animate_light_direction)
        //.add_system(rotate)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut main_transform = Transform::from_xyz(10.0, 10.0, 0.0)
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);

    //let mut second_transform = main_transform.clone();
    //second_transform.rotate_local_y(0.64);
    //main_transform.rotate_local_y(-0.64);

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/Dragonite/scene.gltf#Scene0"),
        ..default()
    });

    //second window
    let second_window = commands
        .spawn(Window {
            title: "Second window".to_owned(),
            ..default()
        })
        .id();


    //ship
    let ship = commands.spawn((
        TransformBundle{
            local: main_transform,
            ..default()
        },
        FlyCamera::default()
        // from_xyz(10.0,10.0,0.0)
            // .looking_at(Vec3::new(0.0,0.0,0.0), Vec3::Y),
    )).id();

    //first window camera
    let main_camera = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_rotation(Quat::from_rotation_y(1.28)),
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
        },
    )).id();


    //second window camera
    let second_camera = commands.spawn((
        Camera3dBundle {
            camera: Camera {
                target: RenderTarget::Window(WindowRef::Entity(second_window)),
                ..default()
            },
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
        },
    )).id();

    commands.entity(ship).add_child(main_camera);
    commands.entity(ship).add_child(second_camera);

    // });



    let skybox_handle = asset_server.load(CUBEMAPS[0].0);
    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

fn rotate(
    mut commands: Commands,
    time: Res<Time>,
    mut parents_query: Query<(Entity, &Children)>,
    mut transform_query: Query<&mut Transform>,
) {
    for (parent, children) in &mut parents_query {
        if let Ok(mut transform) = transform_query.get_mut(parent) {
            transform.rotate_z(-PI / 2. * time.delta_seconds());
        }

        // To iterate through the entities children, just treat the Children component as a Vec
        // Alternatively, you could query entities that have a Parent component
        for child in children {
            if let Ok(mut transform) = transform_query.get_mut(*child) {
                transform.rotate_z(PI * time.delta_seconds());
            }
        }

        // To demonstrate removing children, we'll remove a child after a couple of seconds.
        if time.elapsed_seconds() >= 2.0 && children.len() == 2 {
            let child = children.last().unwrap();
            //commands.entity(*child).despawn_recursive();
        }

        if time.elapsed_seconds() >= 4.0 {
            // This will remove the entity from its parent's list of children, as well as despawn
            // any children the entity has.
            // commands.entity(parent).despawn_recursive();
        }
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "9509a0f8-3c05-48ee-a13e-a93226c7f488"]
struct CubemapMaterial {
    base_color_texture: Option<Handle<Image>>,
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
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let base_color_texture = self
            .base_color_texture
            .as_ref()
            .ok_or(AsBindGroupError::RetryNextUpdate)?;
        let image = images
            .get(base_color_texture)
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


fn cycle_cubemap_asset(
    time: Res<Time>,
    mut next_swap: Local<f32>,
    mut cubemap: ResMut<Cubemap>,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {
    let now = time.elapsed_seconds();
    if *next_swap == 0.0 {
        *next_swap = now + CUBEMAP_SWAP_DELAY;
        return;
    } else if now < *next_swap {
        return;
    }
    *next_swap += CUBEMAP_SWAP_DELAY;

    let supported_compressed_formats =
        CompressedImageFormats::from_features(render_device.features());

    let mut new_index = cubemap.index;
    for _ in 0..CUBEMAPS.len() {
        new_index = (new_index + 1) % CUBEMAPS.len();
        if supported_compressed_formats.contains(CUBEMAPS[new_index].1) {
            break;
        }
        info!("Skipping unsupported format: {:?}", CUBEMAPS[new_index]);
    }

    // Skip swapping to the same texture. Useful for when ktx2, zstd, or compressed texture support
    // is missing
    if new_index == cubemap.index {
        return;
    }

    cubemap.index = new_index;
    cubemap.image_handle = asset_server.load(CUBEMAPS[cubemap.index].0);
    cubemap.is_loaded = false;
}


fn asset_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cubemap_materials: ResMut<Assets<CubemapMaterial>>,
    mut cubemap: ResMut<Cubemap>,
    cubes: Query<&Handle<CubemapMaterial>>,
) {
    if !cubemap.is_loaded
        && asset_server.get_load_state(cubemap.image_handle.clone_weak()) == LoadState::Loaded
    {
        info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);

        let mut image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(
                image.texture_descriptor.size.height / image.texture_descriptor.size.width,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        // spawn cube
        commands.spawn(MaterialMeshBundle::<CubemapMaterial> {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 10000.0 })),
            material: cubemap_materials.add(CubemapMaterial {
                base_color_texture: Some(cubemap.image_handle.clone_weak()),
            }),
            ..default()
        });

        cubemap.is_loaded = true;
    }
}
