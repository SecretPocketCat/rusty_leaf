use crate::drag::DragGroup;
use crate::GameState;
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::text::Text2dSize;
use bevy::window::{WindowCreated, WindowResized};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_interact_2d::InteractionSource;
use bevy_pixel_camera::{PixelCameraBundle, PixelProjection};
use iyes_loopless::prelude::AppLooplessStateExt;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ViewScale(1))
            .add_startup_system(setup_pixel_cam)
            .add_system_to_stage(CoreStage::PostUpdate, set_z)
            .add_system(on_resize)
            .add_system(on_scale_change)
            .add_system(on_text_2d_added);
    }
}

pub const VIEW_SIZE: Vec2 = Vec2::new(320., 180.);
pub const VIEW_PADDING: f32 = 5.;
pub const VIEW_EXTENDS: Vec2 = Vec2::new(VIEW_SIZE.x / 2., VIEW_SIZE.y / 2.);
pub const PADDED_VIEW_EXTENDS: Vec2 =
    Vec2::new(VIEW_EXTENDS.x - VIEW_PADDING, VIEW_EXTENDS.y - VIEW_PADDING);
pub const COL_DARK: Color = Color::rgb(0.2706, 0.2392, 0.2784);
pub const COL_DARKER: Color = Color::rgb(0.1137, 0.0941, 0.0706);
pub const COL_LIGHT: Color = Color::rgb(0.9372, 0.847, 0.7294);
pub const COL_OUTLINE_HIGHLIGHTED: Color = Color::rgb(0.9647, 0.502, 0.2431);
pub const COL_OUTLINE_HIGHLIGHTED_2: Color = Color::rgb(0.6745, 0.2352, 0.1333);
pub const COL_OUTLINE_HOVERED_DRAG: Color = Color::rgb(0.3333, 0.5333, 0.247);

#[derive(Component)]
pub struct MainCam;

#[derive(Component, Clone, Copy)]
pub enum ZIndex {
    Bg = 0,
    Shopkeep,
    BgShop,
    Character,
    FirePit,
    Cauldron,
    Fire,
    Grid,
    OrderTooltip,
    Explosion,
    Piece,
    Card,
    Tooltip,
    Dragged,
}

impl From<ZIndex> for f32 {
    fn from(z_index: ZIndex) -> Self {
        z_index as u8 as f32
    }
}

#[derive(Deref, DerefMut)]
pub struct ViewScale(pub u8);

#[derive(Component)]
pub struct ScaledView;

fn setup_pixel_cam(mut cmd: Commands) {
    cmd.spawn_bundle(PixelCameraBundle::from_zoom(1))
        .insert(MainCam)
        .insert(InteractionSource {
            groups: vec![
                DragGroup::Card.into(),
                DragGroup::Piece.into(),
                DragGroup::Cauldron.into(),
                DragGroup::Fire.into(),
                DragGroup::Grid.into(),
                DragGroup::GridPieces.into(),
                DragGroup::GridSection.into(),
            ],
            ..Default::default()
        });
}

// can use this e.g. for particles
// fn setup_render_tex(mut cmd: Commands, mut images: ResMut<Assets<Image>>) {
//     let size = Extent3d {
//         width: VIEW_SIZE.x as u32,
//         height: VIEW_SIZE.y as u32,
//         ..default()
//     };

//     // view texture
//     let mut view_img = Image {
//         texture_descriptor: TextureDescriptor {
//             label: None,
//             size,
//             dimension: TextureDimension::D2,
//             format: TextureFormat::Rgba8UnormSrgb,
//             mip_level_count: 1,
//             sample_count: 1,
//             usage: TextureUsages::TEXTURE_BINDING
//                 | TextureUsages::COPY_DST
//                 | TextureUsages::RENDER_ATTACHMENT,
//         },
//         ..default()
//     };

//     // fill image.data with zeroes
//     view_img.resize(size);
//     let image_handle = images.add(view_img);
//     let rescale_pass_layer = RenderLayers::layer(1);

//     cmd.spawn_bundle(Camera2dBundle {
//         camera: Camera {
//             priority: -1,
//             target: RenderTarget::Image(image_handle.clone()),
//             ..default()
//         },
//         ..default()
//     });

//     cmd.spawn_bundle(SpriteBundle {
//         texture: image_handle.clone(),
//         transform: Transform::from_scale(Vec2::splat(2.).extend(1.)),
//         ..default()
//     })
//     .insert(ScaledView)
//     .insert(rescale_pass_layer);

//     cmd.spawn_bundle(Camera2dBundle::default())
//         .insert(rescale_pass_layer);
// }

fn on_resize(
    mut resize_evr: EventReader<WindowResized>,
    mut scale: ResMut<ViewScale>,
    mut pixel_cam_q: Query<(
        &mut PixelProjection,
        &mut Camera,
    )>,
    windows: Res<Windows>,
) {
    let mut scale_changed = false;
    let window_resized = !resize_evr.is_empty();

    for ev in resize_evr.iter() {
        let new_scale = (Vec2::new(ev.width, ev.height) / VIEW_SIZE)
            .floor()
            .min_element() as u8;

        if scale.0 != new_scale {
            scale.0 = new_scale;
            scale_changed = true;
        }
    }

    for ( mut projection, mut cam) in pixel_cam_q.iter_mut() {
        if window_resized || scale_changed {
            if let RenderTarget::Window(win_id) = cam.target 
                && let Some(win) = windows.get(win_id) {
                projection.zoom = scale.0 as i32;
    
                let scale_factor = win.scale_factor() as f32;
                let target_size = UVec2::new(win.physical_width(), win.physical_height());
                let size = VIEW_SIZE * scale.0 as f32 * scale_factor;
                let pos = ((target_size.as_vec2()) - size) / 2.;
 
                cam.viewport = Some(Viewport {
                    physical_position: pos.as_uvec2(),
                    physical_size: size.as_uvec2(),
                    ..default()
                });
            }
        }
    }
}

fn on_scale_change(
    scale: Res<ViewScale>,
    mut scaled_view_q: Query<&mut Transform, With<ScaledView>>,
) {
    if scale.is_changed() {
        for mut t in scaled_view_q.iter_mut() {
            t.scale = Vec2::splat(scale.0 as f32).extend(1.);
        }
    }
}

fn on_text_2d_added(mut added_q: Query<(&mut Text, &mut Transform), Added<Text2dSize>>) {
    let factor = 20.;
    for (mut txt, mut t) in added_q.iter_mut() {
        t.scale = Vec2::splat(1. / factor).extend(1.);
        for s in txt.sections.iter_mut() {
            s.style.font_size *= factor;
        }
    }
}

fn set_z(mut z_query: Query<(&ZIndex, &mut Transform), Or<(Changed<ZIndex>, Changed<Transform>)>>) {
    for (z, mut t) in z_query.iter_mut() {
        t.translation.z = (*z).into();
    }
}
