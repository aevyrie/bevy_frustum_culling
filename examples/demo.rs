use bevy::{prelude::*, render::camera::Camera};

use bevy_frustum_culling::*;
use bevy_mod_bounding::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            vsync: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BoundingVolumePlugin::<AxisAlignedBB>::default())
        .add_plugin(FrustumCullingPlugin::<AxisAlignedBB>::default())
        .add_startup_system(setup.system())
        .add_system(rotation_system.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_path = "models/waterbottle/WaterBottle.gltf#Mesh0/Primitive0";
    let _scenes: Vec<HandleUntyped> = asset_server.load_folder("models").unwrap();
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(0.8, 0.7, 0.6),
        ..Default::default()
    });
    let mesh_handle = asset_server.get_handle(mesh_path);

    commands
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(15.0, 15.0, 15.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .spawn(PerspectiveCameraBundle {
            camera: Camera {
                name: Some("Secondary".to_string()),
                ..Default::default()
            },
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with(FrustumCulling)
        .with(Rotator)
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: cube_handle,
                material: cube_material_handle,
                ..Default::default()
            });
        })
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });

    for x in -20..20 {
        for y in -20..20 {
            for z in -20..20 {
                commands
                    .spawn(PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32 * 1.0,
                            y as f32 * 1.0,
                            z as f32 * 1.0,
                        )),
                        ..Default::default()
                    })
                    .with(AddBoundingVolume::<AxisAlignedBB>::default());
            }
        }
    }
}

struct Rotator;

fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in query.iter_mut() {
        let rot_y = Quat::from_rotation_y(time.seconds_since_startup() as f32 * 2.0);
        *transform = Transform::from_rotation(rot_y);
    }
}
