use bevy::prelude::*;

pub struct GameInputPlugin;
impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_kb_input);
    }
}

fn handle_kb_input(kb_input: Res<Input<KeyCode>>) {
    if kb_input.just_pressed(KeyCode::Escape) {
        // yep, this's lazy
        // todo: redo
        web_sys::window().unwrap().location().reload().unwrap();
        // lvl_evw.send(LevelEv::LevelOver { won: false });
        // cmd.insert_resource(NextState(GameState::Playing));
    }
}
