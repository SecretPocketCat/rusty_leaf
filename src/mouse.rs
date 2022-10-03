use bevy::{prelude::*, render::camera::RenderTarget};

use crate::render::MainCam;

pub struct MousePlugin;
impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .add_system(store_cursor_pos);
    }
}

#[derive(Default)]
pub struct CursorWorldPosition(pub Vec2);

#[allow(clippy::only_used_in_recursion)]
fn store_cursor_pos(
    wnds: Res<Windows>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut cursor_pos: ResMut<CursorWorldPosition>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (cam, camera_transform) = camera_q.single();

    if let RenderTarget::Window(window_id) = cam.target {
        let win = wnds.get(window_id).unwrap();

        // check if the cursor is inside the window and get its position
        if let Some(screen_pos) = win.cursor_position() {
            let win_size = Vec2::new(win.width() as f32, win.height() as f32);
            let viewport_scale = cam.viewport.as_ref().map_or(Vec2::ONE, |viewport| {
                Vec2::new(
                    viewport.physical_size.x as f32 / win.physical_width() as f32,
                    viewport.physical_size.y as f32 / win.physical_height() as f32,
                )
            });

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = ((screen_pos / win_size) * 2.0 - Vec2::ONE);
            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * cam.projection_matrix().inverse();
            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            cursor_pos.0 = world_pos.truncate() / viewport_scale;
        }
    }
}
