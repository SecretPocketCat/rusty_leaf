#![feature(int_roundings)]
#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::cast_precision_loss,
    clippy::needless_update,
    // jam-code only!
    dead_code
)]

mod anim;
mod assets;
mod board;
mod card;
mod cauldron;
mod coords;
mod drag;
mod level;
mod mouse;
mod piece;
mod render;
mod tile_placement;

use crate::tile_placement::TilePlacementPlugin;
use anim::AnimationPlugin;
use assets::AssetsPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use bevy_interact_2d::{drag::DragPlugin, InteractionDebugPlugin};
use bevy_prototype_lyon::prelude::ShapePlugin;
use card::{Card, CardPlugin, Ingredient};
use cauldron::CauldronPlugin;
use coords::CoordsPlugin;
use drag::DragPlugin as GameDragPlugin;
use iyes_loopless::prelude::AppLooplessStateExt;
use level::LevelPlugin;
use mouse::MousePlugin;
use render::RenderPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub use render::WINDOW_SIZE;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AssetsPlugin)
            .add_plugin(RenderPlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .register_inspectable::<Card>()
            .register_inspectable::<Ingredient>()
            // .register_inspectable::<TileCoords>()
            // .add_system(log_coords)
            // .add_plugin(InteractionPlugin)
            .add_plugin(InteractionDebugPlugin)
            .add_plugin(DragPlugin)
            .add_plugin(GameDragPlugin)
            .add_plugin(AnimationPlugin)
            .add_plugin(ShapePlugin)
            .add_plugin(TilePlacementPlugin)
            .add_plugin(LevelPlugin)
            .add_plugin(CardPlugin)
            .add_plugin(CauldronPlugin)
            .add_plugin(CoordsPlugin)
            .add_plugin(MousePlugin);
    }
}
