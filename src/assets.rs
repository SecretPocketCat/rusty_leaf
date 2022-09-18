use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::Loading)
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Playing)
                    .with_collection::<AudioAssets>()
                    .with_collection::<Sprites>(),
            );
    }
}

#[derive(AssetCollection)]
pub struct AudioAssets {
    // #[asset(path = "audio/background.ogg")]
    // single_file: Handle<AudioSource>,
}

#[derive(AssetCollection)]
pub struct Sprites {
    // Any file that can be loaded and turned into a texture atlas
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 6, rows = 1))]
    #[asset(path = "sprites/veggies_sheet.png")]
    pub ingredients: Handle<TextureAtlas>,

    #[asset(path = "sprites/card.png")]
    pub card: Handle<Image>,

    #[asset(path = "sprites/bg.png")]
    pub bg: Handle<Image>,

    #[asset(path = "sprites/parchment.png")]
    pub parchment: Handle<Image>,
    // // A collection of asset files loaded to typed asset handles
    // #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    // files_typed: Vec<Handle<Image>>,
}
