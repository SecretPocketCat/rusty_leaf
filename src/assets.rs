use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

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
                    .with_collection::<Sprites>()
                    .with_collection::<Fonts>(),
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
    #[asset(path = "sprites/card.png")]
    pub card: Handle<Image>,

    #[asset(path = "sprites/card_outline.png")]
    pub card_outline: Handle<Image>,

    #[asset(path = "sprites/bg_1.png")]
    pub bg: Handle<Image>,

    #[asset(path = "sprites/bg_2.png")]
    pub bg_shop: Handle<Image>,

    #[asset(path = "sprites/parchment.png")]
    pub parchment: Handle<Image>,

    #[asset(path = "sprites/hint_tooltip.png")]
    pub hint_tooltip: Handle<Image>,

    #[asset(path = "sprites/order_tooltip.png")]
    pub order_tooltip: Handle<Image>,

    #[asset(path = "sprites/progress_tooltip.png")]
    pub progress_tooltip: Handle<Image>,

    #[asset(path = "sprites/progress_bar.png")]
    pub progress_bar: Handle<Image>,

    #[asset(path = "sprites/title_tooltip.png")]
    pub title_tooltip: Handle<Image>,

    #[asset(texture_atlas(tile_size_x = 28., tile_size_y = 24., columns = 6, rows = 1))]
    #[asset(path = "sprites/ferris.png")]
    pub ferris: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 24., tile_size_y = 24., columns = 4, rows = 8))]
    #[asset(path = "sprites/characters.png")]
    pub characters: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 6, rows = 1))]
    #[asset(path = "sprites/veggies_sheet.png")]
    pub ingredients: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 37., tile_size_y = 30., columns = 2, rows = 1))]
    #[asset(path = "sprites/cauldron_sheet.png")]
    pub cauldron: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 37., tile_size_y = 30., columns = 2, rows = 1))]
    #[asset(path = "sprites/cauldron_sheet_outline.png")]
    pub cauldron_outline: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 17., tile_size_y = 24., columns = 8, rows = 2))]
    #[asset(path = "sprites/fire_sheet.png")]
    pub fire: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 32., columns = 2, rows = 1))]
    #[asset(path = "sprites/firepit_sheet.png")]
    pub firepit: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 32., columns = 2, rows = 1))]
    #[asset(path = "sprites/firepit_sheet_outline.png")]
    pub firepit_outline: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 32., columns = 6, rows = 1))]
    #[asset(path = "sprites/shop_smoke_sheet.png")]
    pub shop_smoke: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 10, rows = 1))]
    #[asset(path = "sprites/explosion.png")]
    pub explosion: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 4, rows = 4))]
    #[asset(path = "sprites/crosses.png")]
    pub crosses: Handle<TextureAtlas>,
    //
    // // A collection of asset files loaded to typed asset handles
    // #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    // files_typed: Vec<Handle<Image>>,
}

#[derive(AssetCollection)]
pub struct Fonts {
    #[asset(path = "fonts/m3x6.ttf")]
    pub tooltip: Handle<Font>,
}
