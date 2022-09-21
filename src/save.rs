use bevy::prelude::*;
use iyes_loopless::prelude::*;
use web_sys;

use crate::level::{CurrentLevel, LevelEv};

pub struct SavePlugin;
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(store_level)
            .add_startup_system(restore_level);
    }
}

const SAVE_KEY: &str = "rusty_lvl";

fn store_level(mut lvl_evr: EventReader<LevelEv>, lvl: Res<CurrentLevel>) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelOut = ev {
            write_save(lvl.level_index);
        }
    }
}

fn restore_level(mut cmd: Commands) {
    let lvl = read_save();
    cmd.insert_resource(CurrentLevel::new(lvl, false));
}

// todo: handle non-wasm, also error handling...
fn write_save(level: usize) {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    storage.set_item(SAVE_KEY, &level.to_string()).unwrap();
}

// todo: handle non-wasm, also error handling...
fn read_save() -> usize {
    // return 0;
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

    match storage.get_item(SAVE_KEY).unwrap() {
        Some(val) => str::parse(&val).unwrap_or(0),
        None => 0,
    }
}
