//! Loads and renders a glTF file as a scene.

use std::f32::consts::*;
use bevy_fly_camera::{FlyCamera,FlyCameraPlugin};

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    render::camera::RenderTarget,
    window::WindowRef,
};

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup)
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
