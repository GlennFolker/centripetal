use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_mod_picking::{backend::prelude::*, picking_core::PickSet};

pub struct Avian2dBackend;
impl Plugin for Avian2dBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_hits.in_set(PickSet::Backend));
    }
}

pub fn update_hits(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    transforms: Query<&GlobalTransform>,
    parents: Query<&Parent>,
    spatial_query: Option<Res<SpatialQueryPipeline>>,
    mut output: EventWriter<PointerHits>,
) {
    let Some(spatial_query) = spatial_query else { return };
    for (&id, loc) in &pointers {
        let Some(loc) = loc.location() else { continue };
        let Some((camera_entity, camera, camera_trns, camera_ortho)) =
            cameras.iter().filter(|&(_, camera, _, _)| camera.is_active).find(|&(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match primary_window.get_single() {
                        Ok(w) => w,
                        Err(..) => return false,
                    }))
                    .is_some_and(|target| target == loc.target)
            })
        else {
            continue;
        };

        let Some(pos) = camera.viewport_to_world_2d(camera_trns, loc.position) else {
            continue;
        };

        let mut picks = Vec::new();
        spatial_query.point_intersections_callback(pos, SpatialQueryFilter::default(), |e| {
            let mut target = e;
            let Some(trns) = (loop {
                match transforms.get(target) {
                    Ok(&trns) => break Some(trns),
                    Err(..) => {
                        target = match parents.get(target).map(Parent::get) {
                            Ok(parent) => parent,
                            Err(..) => break None,
                        }
                    }
                }
            }) else {
                return false
            };

            let trns = trns.translation();
            picks.push((e, HitData::new(camera_entity, -camera_ortho.near - trns.z, Some(pos.extend(trns.z)), None)));
            false
        });

        output.send(PointerHits::new(id, picks, camera.order as f32));
    }
}
