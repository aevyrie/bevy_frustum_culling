use bevy::{prelude::*, render::camera::Camera, tasks::ComputeTaskPool};
pub use bevy_mod_bounding::*;
use std::marker::PhantomData;

#[derive(Default)]
pub struct FrustumCullingPlugin<T>(PhantomData<T>);
impl<T: 'static + Send + Sync + BoundingVolume> Plugin for FrustumCullingPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            frustum_culling::<T>
                .system()
                .after(BoundingSystem::UpdateBounds)
                .after(bevy::transform::TransformSystem::TransformPropagate)
                .before(bevy::render::RenderSystem::VisibleEntities),
        );
    }
}

pub struct FrustumCulling;

fn frustum_culling<T: 'static + BoundingVolume + Send + Sync>(
    pool: Res<ComputeTaskPool>,
    camera_query: Query<(&Camera, &GlobalTransform), With<FrustumCulling>>,
    mut bound_vol_query: Query<(&T, &GlobalTransform, &mut Visible)>,
) {
    // TODO: only compute frustum on camera change. Can store in a frustum component.
    for (camera, camera_position) in camera_query.iter() {
        let ndc_to_world: Mat4 =
            camera_position.compute_matrix() * camera.projection_matrix.inverse();
        // Near/Far, Top/Bottom, Left/Right
        let nbl_world = ndc_to_world.project_point3(Vec3::new(-1.0, -1.0, -1.0));
        let nbr_world = ndc_to_world.project_point3(Vec3::new(1.0, -1.0, -1.0));
        let ntl_world = ndc_to_world.project_point3(Vec3::new(-1.0, 1.0, -1.0));
        let fbl_world = ndc_to_world.project_point3(Vec3::new(-1.0, -1.0, 1.0));
        let ftr_world = ndc_to_world.project_point3(Vec3::new(1.0, 1.0, 1.0));
        let ftl_world = ndc_to_world.project_point3(Vec3::new(-1.0, 1.0, 1.0));
        let fbr_world = ndc_to_world.project_point3(Vec3::new(1.0, -1.0, 1.0));
        let ntr_world = ndc_to_world.project_point3(Vec3::new(1.0, 1.0, -1.0));
        // Compute plane normals
        let near_plane = (nbr_world - nbl_world)
            .cross(ntl_world - nbl_world)
            .normalize();
        let far_plane = (fbr_world - ftr_world)
            .cross(ftl_world - ftr_world)
            .normalize();
        let top_plane = (ftl_world - ftr_world)
            .cross(ntr_world - ftr_world)
            .normalize();
        let bottom_plane = (fbl_world - nbl_world)
            .cross(nbr_world - nbl_world)
            .normalize();
        let right_plane = (ntr_world - ftr_world)
            .cross(fbr_world - ftr_world)
            .normalize();
        let left_plane = (ntl_world - nbl_world)
            .cross(fbl_world - nbl_world)
            .normalize();

        let frustum_plane_list = [
            (nbl_world, left_plane),
            (ftr_world, right_plane),
            (nbl_world, bottom_plane),
            (ftr_world, top_plane),
            (nbl_world, near_plane),
            (ftr_world, far_plane),
        ];

        // If a bounding volume is entirely outside of any camera frustum plane, it is not visible.
        bound_vol_query.par_for_each_mut(
            &pool,
            32,
            |(bound_vol, bound_vol_position, mut visible)| {
                for (plane_point, plane_normal) in frustum_plane_list.iter() {
                    if bound_vol.outside_plane(bound_vol_position, *plane_point, *plane_normal) {
                        visible.is_visible = false;
                        return;
                    }
                }
                visible.is_visible = true;
            },
        );
    }
}
