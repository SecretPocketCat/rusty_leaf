use crate::{
    assets::Sprites,
    level::{tween_on_level_ev, EvTween, LevelEv, LevelEventTweenType},
    render::ZIndex,
    GameState,
};
use bevy::prelude::*;
use bevy_tweening::EaseFunction;
use iyes_loopless::prelude::AppLooplessStateExt;

pub struct WinPlugin;
impl Plugin for WinPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WinEv>()
            .add_exit_system(GameState::Loading, setup)
            .add_enter_system(GameState::Won, on_win_in)
            .add_exit_system(GameState::Won, on_win_out)
            .add_system(tween_on_level_ev::<WinEv>);
    }
}

#[derive(PartialEq, Eq)]
pub enum WinEv {
    In,
    Out,
}

fn setup(mut cmd: Commands, sprites: Res<Sprites>) {
    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.win.clone(),
        transform: Transform::from_xyz(0., 800., 0.),
        ..default()
    })
    .insert(ZIndex::Tooltip)
    .insert(
        EvTween::new(
            LevelEventTweenType::MoveByY(-800.),
            WinEv::In,
            WinEv::Out,
            1200,
        )
        .with_delay_in(1000)
        .with_ease_in(EaseFunction::CircularOut)
        .with_ease_out(EaseFunction::CircularOut),
    )
    .insert(Name::new("won"));
}

fn on_win_in(mut evw: EventWriter<WinEv>) {
    evw.send(WinEv::In);
}

fn on_win_out(mut evw: EventWriter<WinEv>) {
    evw.send(WinEv::Out);
}
