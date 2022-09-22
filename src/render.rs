use crate::drag::DragGroup;
use bevy::prelude::*;
use bevy_interact_2d::InteractionSource;
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(scale_sprites)
            .add_system_to_stage(CoreStage::PostUpdate, set_z);
    }
}

pub const WINDOW_SIZE: Vec2 = Vec2::new(1280., 720.);
pub const SCALE_MULT: f32 = 4.; // todo: resource and handled on win resize?
pub const COL_DARK: Color = Color::rgb(0.2706, 0.2392, 0.2784);
pub const COL_LIGHT: Color = Color::rgb(0.9372, 0.847, 0.7294);
pub const COL_OUTLINE_HIGHLIGHTED: Color = Color::rgb(0.9647, 0.502, 0.2431);
pub const COL_OUTLINE_HIGHLIGHTED_2: Color = Color::rgb(0.6745, 0.2352, 0.1333);
pub const COL_OUTLINE_HOVERED_DRAG: Color = Color::rgb(0.3333, 0.5333, 0.247);

#[derive(Component)]
pub struct MainCam;

#[derive(Component)]
pub struct NoRescale;

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
    Piece,
    Card,
    Tooltip,
    Dragged,
}

impl From<ZIndex> for f32 {
    fn from(z_index: ZIndex) -> Self {
        z_index as u8 as f32 / 10.
    }
}

fn setup(mut cmd: Commands) {
    cmd.spawn_bundle(Camera2dBundle::default())
        // replace default cam pos to shift the tilegrid so that it's still centered in the world 'cause I can't be bothered to fix the coordinates for shifting it off-center
        // .insert(Transform::from_xyz(260., 0., 999.))
        .insert(MainCam)
        .insert(InteractionSource {
            groups: vec![
                DragGroup::Card.into(),
                DragGroup::Piece.into(),
                DragGroup::Cauldron.into(),
                DragGroup::Fire.into(),
                DragGroup::Grid.into(),
                DragGroup::GridPieces.into(),
            ],
            ..Default::default()
        });
}

fn scale_sprites(
    mut sprite_q: Query<
        &mut Transform,
        (
            Without<NoRescale>,
            Or<(Added<Sprite>, Added<TextureAtlasSprite>)>,
        ),
    >,
) {
    for mut t in sprite_q.iter_mut() {
        t.scale = Vec2::splat(SCALE_MULT).extend(1.0);
    }
}

fn set_z(mut z_query: Query<(&ZIndex, &mut Transform), Or<(Changed<ZIndex>, Changed<Transform>)>>) {
    for (z, mut t) in z_query.iter_mut() {
        t.translation.z = (*z).into();
    }
}
