use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::DrawMode;
use bevy_tweening::lens::{SpriteColorLens, TransformPositionLens, TransformScaleLens};
use bevy_tweening::*;
use std::time::Duration;

pub struct GameTweenPlugin;
impl Plugin for GameTweenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(on_tween_completed)
            .add_system(component_animator_system::<DrawMode>)
            .add_system(component_animator_system::<TextureAtlasSprite>)
            // todo: is this needed?
            // .add_system(component_animator_system::<Text>)
            .add_system(on_fade_hiearchy_set_added)
            .add_system(on_fadeable_child_added)
            .add_system(
                fade_hierarchy
                    .after(on_fade_hiearchy_set_added)
                    .after(on_fadeable_child_added),
            );
    }
}

#[repr(u64)]
#[derive(Clone)]
pub enum TweenDoneAction {
    None = 0,
    DespawnRecursive = 1,
}

impl From<u64> for TweenDoneAction {
    fn from(val: u64) -> Self {
        unsafe { ::std::mem::transmute(val) }
    }
}

impl From<TweenDoneAction> for u64 {
    fn from(val: TweenDoneAction) -> Self {
        val as u64
    }
}

#[derive(Component)]
pub struct FadeHierarchy {
    fade_in: bool,
    duration_ms: u64,
    text_color: Color,
    done_action: Option<TweenDoneAction>,
}

impl FadeHierarchy {
    pub fn new(fade_in: bool, duration_ms: u64, text_color: Color) -> Self {
        Self {
            fade_in,
            duration_ms,
            text_color,
            done_action: None,
        }
    }

    pub fn with_done_action(mut self, action: TweenDoneAction) -> Self {
        self.done_action = Some(action);
        self
    }
}

#[derive(Component, Default)]
pub struct FadeHierarchySet(HashSet<FadeChild>);

#[derive(Bundle)]
pub struct FadeHierarchyBundle {
    fade_hierarchy: FadeHierarchy,
    set: FadeHierarchySet,
}

impl FadeHierarchyBundle {
    pub fn new(fade_in: bool, duration_ms: u64, text_color: Color) -> Self {
        Self {
            fade_hierarchy: FadeHierarchy::new(fade_in, duration_ms, text_color),
            set: Default::default(),
        }
    }

    pub fn with_done_action(mut self, action: TweenDoneAction) -> Self {
        self.fade_hierarchy.done_action = Some(action);
        self
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FadeChild {
    Sprite(Entity),
    SpriteSheet(Entity),
    Text(Entity),
}

fn on_tween_completed(
    mut commands: Commands,
    mut ev_reader: EventReader<TweenCompleted>,
    entity_q: Query<Entity>,
) {
    for ev in ev_reader.iter() {
        match TweenDoneAction::from(ev.user_data) {
            TweenDoneAction::None => {}
            TweenDoneAction::DespawnRecursive => {
                if entity_q.get(ev.entity).is_ok() {
                    commands.entity(ev.entity).despawn_recursive();
                }
            }
        }
    }
}

fn fade_hierarchy(
    mut cmd: Commands,
    fade_hierarchy_q: Query<(Entity, &FadeHierarchySet, &FadeHierarchy)>,
    entity_q: Query<Entity>,
) {
    for (hierarchy_e, set, fade) in fade_hierarchy_q.iter() {
        let col = if fade.fade_in {
            Color::WHITE
        } else {
            Color::NONE
        };

        for sprite in set.0.iter() {
            match sprite {
                FadeChild::Sprite(e) => {
                    if let Ok(e) = entity_q.get(*e) {
                        cmd.entity(e).insert(get_relative_sprite_color_anim(
                            col,
                            fade.duration_ms,
                            fade.done_action.clone(),
                        ));
                    }
                }
                FadeChild::SpriteSheet(e) => {
                    if let Ok(e) = entity_q.get(*e) {
                        cmd.entity(e).insert(get_relative_spritesheet_color_anim(
                            col,
                            fade.duration_ms,
                            fade.done_action.clone(),
                        ));
                    }
                }
                FadeChild::Text(e) => {
                    if let Ok(e) = entity_q.get(*e) {
                        cmd.entity(e).insert(get_relative_fade_text_anim(
                            fade.text_color,
                            fade.duration_ms,
                            fade.done_action.clone(),
                        ));
                    }
                }
            }
        }

        cmd.entity(hierarchy_e).remove::<FadeHierarchy>();
    }
}

fn on_fadeable_child_added(
    added_q: Query<
        (Entity, Option<&Sprite>, Option<&TextureAtlasSprite>),
        Or<(Added<Sprite>, Added<TextureAtlasSprite>, Added<Text>)>,
    >,
    parent_q: Query<&Parent>,
    mut fade_q: Query<&mut FadeHierarchySet>,
) {
    for (added_e, sprite, spritesheet) in added_q.iter() {
        let mut current_e = added_e;
        let sprite_child = if sprite.is_some() {
            FadeChild::Sprite(added_e)
        } else if spritesheet.is_some() {
            FadeChild::SpriteSheet(added_e)
        } else {
            FadeChild::Text(added_e)
        };

        while let Ok(p) = parent_q.get(current_e) {
            if let Ok(mut fade) = fade_q.get_mut(p.get()) {
                fade.0.insert(sprite_child);
            }

            current_e = p.get();
        }
    }
}

fn on_fade_hiearchy_set_added(
    mut added_q: Query<(Entity, &mut FadeHierarchySet), Added<FadeHierarchySet>>,
    children_q: Query<&Children>,
    fadeable_q: Query<
        (Option<&Sprite>, Option<&TextureAtlasSprite>),
        Or<(With<Sprite>, With<TextureAtlasSprite>, Added<Text>)>,
    >,
) {
    struct Recurse<'s> {
        get_children: &'s dyn Fn(&Recurse, &mut HashSet<FadeChild>, Entity),
    }

    let recurse = Recurse {
        get_children: &|recurse, set, e| {
            if let Ok((sprite, spritesheet)) = fadeable_q.get(e) {
                let sprite_child = if sprite.is_some() {
                    FadeChild::Sprite(e)
                } else if spritesheet.is_some() {
                    FadeChild::SpriteSheet(e)
                } else {
                    FadeChild::Text(e)
                };

                set.insert(sprite_child);
            }

            if let Ok(children) = children_q.get(e) {
                for c in children.iter() {
                    (recurse.get_children)(recurse, set, *c);
                }
            }
        },
    };

    for (e, mut fade) in added_q.iter_mut() {
        (recurse.get_children)(&recurse, &mut fade.0, e);
    }
}

pub fn lerp_color(from: Color, to: Color, ratio: f32) -> Color {
    let start: Vec4 = from.into();
    let end: Vec4 = to.into();
    start.lerp(end, ratio).into()
}

pub fn delay_tween<T: 'static>(tween: Tween<T>, delay_ms: u64) -> Sequence<T> {
    if delay_ms > 0 {
        Delay::new(Duration::from_millis(delay_ms)).then(tween)
    } else {
        Sequence::new([tween])
    }
}

pub fn get_relative_move_anim(
    end_pos: Vec3,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Transform> {
    Animator::new(get_relative_move_tween(end_pos, duration_ms, on_completed))
}

pub fn get_relative_move_tween(
    end_pos: Vec3,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Transform> {
    let mut tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(duration_ms),
        TransformRelativePositionLens {
            start: Vec3::ZERO,
            end: end_pos,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub fn get_move_anim(
    start_pos: Vec3,
    end_pos: Vec3,
    duration_ms: u64,
    ease: EaseFunction,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Transform> {
    Animator::new(get_move_tween(
        start_pos,
        end_pos,
        duration_ms,
        ease,
        on_completed,
    ))
}

pub fn get_relative_move_by_anim(
    move_by: Vec3,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Transform> {
    Animator::new(get_relative_move_by_tween(
        move_by,
        duration_ms,
        EaseFunction::QuadraticInOut,
        on_completed,
    ))
}

pub fn get_relative_move_by_tween(
    move_by: Vec3,
    duration_ms: u64,
    ease: EaseFunction,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Transform> {
    let mut tween = Tween::new(
        ease,
        Duration::from_millis(duration_ms),
        TransformRelativeByPositionLens::new(move_by),
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub fn get_move_tween(
    start_pos: Vec3,
    end_pos: Vec3,
    duration_ms: u64,
    ease: EaseFunction,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Transform> {
    let mut tween = Tween::new(
        ease,
        Duration::from_millis(duration_ms),
        TransformPositionLens {
            start: start_pos,
            end: end_pos,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub fn get_relative_sprite_color_anim(
    col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Sprite> {
    Animator::new(get_relative_sprite_color_tween(
        col,
        duration_ms,
        on_completed,
    ))
}

pub fn get_relative_sprite_color_tween(
    col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Sprite> {
    let mut tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(duration_ms),
        SpriteRelativeColorLens {
            start: Color::NONE,
            end: col,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub fn get_relative_fade_text_anim(
    col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Text> {
    Animator::new(get_relative_fade_text_tween(
        col,
        duration_ms,
        EaseFunction::QuadraticInOut,
        on_completed,
    ))
}

pub fn get_relative_fade_text_tween(
    col: Color,
    duration_ms: u64,
    ease: EaseFunction,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Text> {
    let mut tween = Tween::new(
        ease,
        Duration::from_millis(duration_ms),
        TextRelativeColorLens {
            start: Vec::new(),
            end: col,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub fn get_relative_spritesheet_color_anim(
    col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<TextureAtlasSprite> {
    Animator::new(get_relative_fade_spritesheet_tween(
        col,
        duration_ms,
        on_completed,
    ))
}

pub fn get_relative_fade_spritesheet_tween(
    col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<TextureAtlasSprite> {
    let mut tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(duration_ms),
        SpriteSheetRelativeColorLens {
            start: Color::NONE,
            end: col,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub fn get_scale_tween(
    start_scale: Vec3,
    end_scale: Vec3,
    ease: EaseFunction,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Transform> {
    let mut tween = Tween::new(
        ease,
        Duration::from_millis(duration_ms),
        TransformScaleLens {
            start: start_scale,
            end: end_scale,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub fn get_fade_out_sprite_anim(
    start_col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Animator<Sprite> {
    Animator::new(get_fade_out_sprite_tween(
        start_col,
        duration_ms,
        on_completed,
    ))
}

pub fn get_fade_out_sprite_tween(
    start_col: Color,
    duration_ms: u64,
    on_completed: Option<TweenDoneAction>,
) -> Tween<Sprite> {
    let mut tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(duration_ms),
        SpriteColorLens {
            start: start_col,
            end: Color::NONE,
        },
    );

    if let Some(on_completed) = on_completed {
        tween = tween.with_completed_event(on_completed.into());
    }

    tween
}

pub struct ShapeDrawModeColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<DrawMode> for ShapeDrawModeColorLens {
    fn lerp(&mut self, target: &mut DrawMode, ratio: f32) {
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);

        if let DrawMode::Fill(mut fill) = target {
            fill.color = value.into();
        } else if let DrawMode::Stroke(stroke) = target {
            stroke.color = value.into();
        }
    }
}

#[derive(Default)]
pub struct SpriteRelativeColorLens {
    start: Color,
    pub end: Color,
}

impl Lens<Sprite> for SpriteRelativeColorLens {
    fn lerp(&mut self, target: &mut Sprite, ratio: f32) {
        target.color = lerp_color(self.start, self.end, ratio);
    }

    fn update_on_tween_start(&mut self, target: &Sprite) {
        self.start = target.color;
    }
}

#[derive(Default)]
pub struct SpriteSheetRelativeColorLens {
    start: Color,
    pub end: Color,
}

impl Lens<TextureAtlasSprite> for SpriteSheetRelativeColorLens {
    fn lerp(&mut self, target: &mut TextureAtlasSprite, ratio: f32) {
        target.color = lerp_color(self.start, self.end, ratio);
    }

    fn update_on_tween_start(&mut self, target: &TextureAtlasSprite) {
        self.start = target.color;
    }
}

#[derive(Default)]
pub struct TransformRelativeScaleLens {
    pub start: Vec3,
    pub end: Vec3,
}

impl Lens<Transform> for TransformRelativeScaleLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.scale = value;
    }

    fn update_on_tween_start(&mut self, target: &Transform) {
        self.start = target.scale;
    }
}

#[derive(Default)]
pub struct TransformRelativePositionLens {
    start: Vec3,
    pub end: Vec3,
}

impl Lens<Transform> for TransformRelativePositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.translation = value;
    }

    fn update_on_tween_start(&mut self, target: &Transform) {
        self.start = target.translation;
    }
}

#[derive(Default)]
pub struct TransformRelativeByPositionLens {
    start: Vec3,
    end: Vec3,
    pub move_by: Vec3,
}

impl TransformRelativeByPositionLens {
    pub fn new(move_by: Vec3) -> Self {
        Self {
            move_by,
            start: Vec3::ZERO,
            end: Vec3::ZERO,
        }
    }
}

impl Lens<Transform> for TransformRelativeByPositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.translation = value;
    }

    fn update_on_tween_start(&mut self, target: &Transform) {
        self.start = target.translation;
        self.end = target.translation + self.move_by;
    }
}

#[derive(Default)]
pub struct TextRelativeColorLens {
    start: Vec<Color>,
    pub end: Color,
}

impl Lens<Text> for TextRelativeColorLens {
    fn lerp(&mut self, target: &mut Text, ratio: f32) {
        for i in 0..target.sections.len() {
            target.sections[i].style.color = lerp_color(self.start[i], self.end, ratio);
        }
    }

    fn update_on_tween_start(&mut self, target: &Text) {
        self.start = target.sections.iter().map(|s| s.style.color).collect();
    }
}
