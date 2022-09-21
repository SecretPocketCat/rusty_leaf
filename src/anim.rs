use bevy::prelude::*;
use rand::{thread_rng, Rng};
use std::{ops::Range, time::Duration};

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sheet);
    }
}

#[derive(Component)]
pub struct SheetAnimation {
    timer: Timer,
    range: Option<Range<usize>>,
    start_index: Option<usize>,
}

impl SheetAnimation {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(duration_ms), true),
            range: None,
            start_index: None,
        }
    }

    pub fn with_range(mut self, range: Range<usize>, random_start_index: bool) -> Self {
        if random_start_index {
            self.start_index = Some(thread_rng().gen_range(range.clone()));
        }

        self.set_range(range);
        self
    }

    pub fn set_range(&mut self, range: Range<usize>) {
        self.range = Some(range);
    }

    pub fn set_time(&mut self, duration_ms: u64) {
        self.timer = Timer::new(Duration::from_millis(duration_ms), true);
    }
}

fn animate_sheet(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut SheetAnimation,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut anim, mut sprite, texture_atlas_handle) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let (from, to) = if let Some(ref range) = anim.range {
                (range.start, range.end)
            } else {
                (0, texture_atlas.textures.len())
            };

            if let Some(index) = anim.start_index.take() {
                sprite.index = index;
            } else {
                sprite.index = from + (sprite.index + 1) % (to - from);
            }
        }
    }
}

// fn reset_anim_index_on_sheet_change(
//     mut query: Query<(&mut SheetAnimation, &mut TextureAtlasSprite), Changed<Handle<TextureAtlas>>>,
// ) {
//     for (mut anim, mut sprite) in query.iter_mut() {
//         anim.timer.reset();
//         sprite.index = 0;
//     }
// }
