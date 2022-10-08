use bevy::{prelude::*, render::camera::RenderTarget};

use crate::render::MainCam;

pub struct MousePlugin;
impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .init_resource::<CursorTouch>()
            .add_system(store_cursor_pos)
            .add_system(map_touch);
    }
}

pub struct CursorWorldPosition {
    pub position: Vec2,
    pub delta: Vec2,
}

impl Default for CursorWorldPosition {
    fn default() -> Self {
        // start offscreen
        Self {
            position: Vec2::splat(100000.),
            delta: Vec2::ZERO,
        }
    }
}

#[derive(Debug, Default)]
struct CursorTouch {
    id: Option<u64>,
}

#[allow(clippy::only_used_in_recursion)]
fn store_cursor_pos(
    wnds: Res<Windows>,
    touch: Res<Touches>,
    cursor_touch: Res<CursorTouch>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut cursor_pos: ResMut<CursorWorldPosition>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (cam, camera_transform) = camera_q.single();

    if let RenderTarget::Window(window_id) = cam.target {
        let win = wnds.get(window_id).unwrap();

        let pos = if let Some(id) = cursor_touch.id {
            touch.iter().find(|t| t.id() == id).map(|t| t.position())
        } else {
            win.cursor_position()
        };

        // check if the cursor is inside the window and get its position
        if let Some(screen_pos) = pos {
            let win_size = Vec2::new(win.width() as f32, win.height() as f32);
            let viewport_scale = cam.viewport.as_ref().map_or(Vec2::ONE, |viewport| {
                Vec2::new(
                    viewport.physical_size.x as f32 / win.physical_width() as f32,
                    viewport.physical_size.y as f32 / win.physical_height() as f32,
                )
            });

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / win_size) * 2.0 - Vec2::ONE;
            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * cam.projection_matrix().inverse();
            // use it to convert ndc to world-space coordinates & map it to viewport coordinates
            // todo: this only works for a centered viewport
            let pos = ndc_to_world.project_point3(ndc.extend(-1.0)).truncate() / viewport_scale;
            cursor_pos.delta = pos - cursor_pos.position;
            cursor_pos.position = pos;
        }
    }
}

fn map_touch(mut touch_evr: EventReader<TouchInput>, mut cursor: ResMut<CursorTouch>) {
    use bevy::input::touch::TouchPhase;
    for ev in touch_evr.iter() {
        match ev.phase {
            TouchPhase::Started => {
            if cursor.id.is_none() {
                cursor.id = Some(ev.id);}
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if let Some(  id) = cursor.id && id == ev.id {
                    cursor.id = None;
                }
            }
            TouchPhase::Moved => {}
        }
    }
}
