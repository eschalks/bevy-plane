use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::utils::HashMap;

pub struct BitmapFont(HashMap<char, Handle<Image>>);

#[derive(Default)]
pub enum TextAnchor {
    #[default]
    Left,
    Right,
}

#[derive(Component, Default)]
pub struct BitmapText {
    pub text: String,
    pub anchor: TextAnchor,
}

#[derive(Bundle, Default)]
pub struct BitmapTextBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub text: BitmapText,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl BitmapTextBundle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            transform: Transform::from_xyz(x, y, 10.0),
            ..default()
        }
    }

    pub fn with_anchor(mut self, anchor: TextAnchor) -> Self {
        self.text.anchor = anchor;
        self
    }
}

pub fn create_bitmap_font(asset_server: Res<AssetServer>) -> BitmapFont {
    let mut map = HashMap::new();

    for c in '0'..='9' {
        let path = format!("Numbers/number{}.png", c);
        map.insert(c, asset_server.load(&path));
    }

    for c in 'A'..='Z' {
        let path = format!("Letters/letter{}.png", c);
        let handle = asset_server.load(&path);
        map.insert(c, handle.clone());
        map.insert(c.to_ascii_lowercase(), handle);
    }

    BitmapFont(map)
}

pub fn bitmap_font_system(
    mut commands: Commands,
    font: Res<BitmapFont>,
    texts: Query<(Entity, &BitmapText), Changed<BitmapText>>,
    images: Res<Assets<Image>>,
) {
    for (entity, text) in texts.iter() {
        let mut entity = commands.entity(entity);

        entity.despawn_descendants();

        spawn_letters(&mut entity, &font, text, &images);
    }
}

fn spawn_letters(
    entity: &mut EntityCommands,
    font: &Res<BitmapFont>,
    text: &BitmapText,
    images: &Res<Assets<Image>>,
) {
    // TODO: find a way to just store the result of chars() or chars().rev() into the third tuple item
    let (sprite_anchor, direction, text_str) = match &text.anchor {
        TextAnchor::Left => (Anchor::CenterLeft, 1.0, text.text.clone()),
        TextAnchor::Right => (
            Anchor::CenterRight,
            -1.0,
            text.text.chars().rev().collect::<String>(),
        ),
    };

    let mut x: f32 = 0.0;

    entity.with_children(|parent| {
        for c in text_str.chars() {
            let handle = font.0.get(&c);
            if let Some(handle) = handle {
                let width = if let Some(image) = images.get(&handle) {
                    image.size().x
                } else {
                    // 64 is reasonably safe because it's more than the width of the widest character
                    // Usually doesn't matter because we don't render proper pieces of text until the game over screen or the score reaches 10
                    64.0
                };

                parent.spawn_bundle(SpriteBundle {
                    texture: handle.clone(),
                    transform: Transform::from_xyz(x, 0.0, 0.0),
                    sprite: Sprite {
                        anchor: sprite_anchor.clone(),
                        ..default()
                    },
                    ..default()
                });

                x += (width + 4.0) * direction;
            }
        }
    });
}
