#![feature(int_roundings)]
#![feature(let_chains)]
#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::cast_precision_loss,
    clippy::needless_update,
    irrefutable_let_patterns,
    // jam-code only!
    // dead_code
)]

mod anim;
mod assets;
mod board;
mod card;
mod cauldron;
mod coords;
mod customer;
mod drag;
mod highlight;
mod level;
mod list;
mod mouse;
mod order;
mod piece;
mod progress;
mod render;
mod save;
mod tile_placement;
mod tools;
mod tween;
mod win;

use crate::tile_placement::TilePlacementPlugin;
use anim::AnimationPlugin;
use assets::AssetsPlugin;
use bevy::app::App;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_interact_2d::{drag::DragPlugin, InteractionDebugPlugin, InteractionPlugin};
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_tweening::TweeningPlugin;
use card::{Card, CardPlugin, Ingredient};
use cauldron::CauldronPlugin;
use coords::{CoordsPlugin, TileCoords};
use customer::CustomerPlugin;
use drag::DragPlugin as GameDragPlugin;
use highlight::HighlightPlugin;
use input::GameInputPlugin;
use level::LevelPlugin;
use mouse::MousePlugin;
use order::OrderPlugin;
use progress::ProgressPlugin;
use render::RenderPlugin;
mod input;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    Loading,
    // During this State the actual game logic is executed
    Playing,
    Won,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub use render::VIEW_SIZE;
use save::SavePlugin;
use tween::GameTweenPlugin;
use win::WinPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AssetsPlugin)
            .add_plugin(RenderPlugin)
            .add_plugin(DragPlugin)
            .add_plugin(GameDragPlugin)
            .add_plugin(AnimationPlugin)
            .add_plugin(HighlightPlugin)
            .add_plugin(ShapePlugin)
            .add_plugin(TilePlacementPlugin)
            .add_plugin(LevelPlugin)
            .add_plugin(WinPlugin)
            .add_plugin(CardPlugin)
            .add_plugin(CauldronPlugin)
            .add_plugin(CustomerPlugin)
            .add_plugin(ProgressPlugin)
            .add_plugin(CoordsPlugin)
            .add_plugin(MousePlugin)
            .add_plugin(OrderPlugin)
            .add_plugin(TweeningPlugin)
            .add_plugin(GameTweenPlugin)
            .add_plugin(GameInputPlugin)
            .add_plugin(SavePlugin);

        if cfg!(debug_assertions) {
            // app.add_plugin(WorldInspectorPlugin::new());
            // app.register_inspectable::<Card>()
            //     .register_inspectable::<Ingredient>()
            //     .register_inspectable::<TileCoords>();
            // app.add_plugin(InteractionDebugPlugin);
            app.add_plugin(InteractionPlugin);
        } else {
            app.add_plugin(InteractionPlugin);
        }
    }
}
