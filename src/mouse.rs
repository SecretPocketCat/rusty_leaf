// use crate::{game::game_state::UpdatePhase, render::camera::MainCam};
use bevy::{prelude::*, render::camera::RenderTarget};

use crate::render::{MainCam, ViewScale};

pub struct MousePlugin;
impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .add_system(store_cursor_pos);
    }
}

#[derive(Default)]
pub struct CursorWorldPosition(pub Vec2);

// todo: fix - cam viewport not considered
#[allow(clippy::only_used_in_recursion)]
fn store_cursor_pos(
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut cursor_pos: ResMut<CursorWorldPosition>,
    scale: Res<ViewScale>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    if let RenderTarget::Window(window_id) = camera.target {
        let wnd = wnds.get(window_id).unwrap();

        // check if the cursor is inside the window and get its position
        if let Some(screen_pos) = wnd.cursor_position() {
            let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();
            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            cursor_pos.0 = world_pos.truncate() / scale.0 as f32;
        }
    }
}
