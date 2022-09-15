#![feature(int_roundings)]

mod board;
mod coords;
mod mouse;
mod piece;
mod render;
mod tile_placement;

use crate::tile_placement::TilePlacementPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_interact_2d::{drag::DragPlugin, InteractionDebugPlugin, InteractionPlugin};
use bevy_prototype_lyon::prelude::ShapePlugin;
use coords::CoordsPlugin;
use iyes_loopless::prelude::AppLooplessStateExt;
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

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::Playing)
            .add_plugin(RenderPlugin)
            // .add_plugin(WorldInspectorPlugin::new())
            // .register_inspectable::<TileCoords>()
            // .add_system(log_coords)
            // .add_plugin(InteractionPlugin)
            .add_plugin(InteractionDebugPlugin)
            .add_plugin(DragPlugin)
            .add_plugin(ShapePlugin)
            .add_plugin(TilePlacementPlugin)
            .add_plugin(CoordsPlugin)
            .add_plugin(MousePlugin);
    }
}
