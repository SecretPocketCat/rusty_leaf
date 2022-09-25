use crate::drag::DragGroup;
use bevy::prelude::*;
use bevy_interact_2d::InteractionSource;
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_to_stage(CoreStage::PostUpdate, set_z);
    }
}

pub const WINDOW_SIZE: Vec2 = Vec2::new(320., 180.);
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

fn setup(mut cmd: Commands) {
    cmd.spawn_bundle(Camera2dBundle::default())
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

fn set_z(mut z_query: Query<(&ZIndex, &mut Transform), Or<(Changed<ZIndex>, Changed<Transform>)>>) {
    for (z, mut t) in z_query.iter_mut() {
        t.translation.z = (*z).into();
    }
}
