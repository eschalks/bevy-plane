use crate::{GameState, HorizontalVelocity, Player, PlayerShape, WIDTH};
use bevy::ecs::system::EntityCommands;
use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_prototype_lyon::prelude::*;
use ncollide2d::na;
use ncollide2d::na::{Isometry2, Point2, Vector1, Vector2};
use ncollide2d::query::{self, Contact, Proximity};
use ncollide2d::shape::{ConvexPolygon, Cuboid};

const ROCK_WIDTH: f32 = 108.0;
const ROCK_HEIGHT: f32 = 239.0;
const ROCK_MIN_X: f32 = -WIDTH / 2.0 - ROCK_WIDTH;

const ROCK_UP_POINTS: &'static [(f32, f32)] = &[
    (-52.0, -119.5),
    (52.0, -119.5),
    (12.0, 119.5),
    // (-52.0, -119.5),
];

#[derive(Component)]
pub struct CollisionPolygon {
    polygon: ConvexPolygon<f32>,
}

#[derive(Component)]
pub struct Rock;

enum BevyVec {
    V2(Vec2),
    V3(Vec3),
    T((f32, f32)),
}

impl From<Vec2> for BevyVec {
    fn from(v: Vec2) -> Self {
        Self::V2(v)
    }
}

impl From<Vec3> for BevyVec {
    fn from(v: Vec3) -> Self {
        Self::V3(v)
    }
}

impl From<(f32, f32)> for BevyVec {
    fn from(v: (f32, f32)) -> Self {
        Self::T(v)
    }
}

impl From<&(f32, f32)> for BevyVec {
    fn from(v: &(f32, f32)) -> Self {
        Self::T(*v)
    }
}

fn to_vec2<T: Into<BevyVec>>(v: T) -> Vec2 {
    let v: BevyVec = v.into();

    match v {
        BevyVec::V2(v) => Vec2::new(v.x, v.y),
        BevyVec::V3(v) => Vec2::new(v.x, v.y),
        BevyVec::T((x, y)) => Vec2::new(x, y),
    }
}

fn to_vector2<T: Into<BevyVec>>(v: T) -> Vector2<f32> {
    let v = to_vec2(v);
    Vector2::new(v.x, v.y)
}

fn to_point2<T: Into<BevyVec>>(v: T) -> Point2<f32> {
    let v = to_vector2(v);
    Point2::new(v.x, v.y)
}

pub fn collision_system(
    player_query: Query<(&Player, &Transform)>,
    rock_query: Query<(&CollisionPolygon, &Transform), With<Rock>>,
    mut state: ResMut<State<GameState>>,
) {
    for (player, player_transform) in player_query.iter() {
        let (_, player_angle) = player_transform.rotation.to_axis_angle();

        for (rock_polygon, rock_transform) in rock_query.iter() {
            if is_rock_collision(
                player_transform.translation,
                &player.shape,
                player_angle,
                rock_transform,
                rock_polygon,
            ) {
                state.set(GameState::GameOver).unwrap();
            }
        }
    }
}

fn is_rock_collision(
    player_pos: Vec3,
    player_shape: &PlayerShape,
    player_angle: f32,
    rock_transform: &Transform,
    rock_polygon: &CollisionPolygon,
) -> bool {
    let rock_translation = rock_transform.translation;

    let rock_pos = Isometry2::new(to_vector2(rock_translation), na::zero());
    let player_iso = Isometry2::new(to_vector2(player_pos), player_angle);

    let p = query::proximity(
        &rock_pos,
        &rock_polygon.polygon,
        &player_iso,
        player_shape,
        1.0,
    );

    match p {
        Proximity::Intersecting => true,
        _ => false,
    }
}

pub fn rock_system(mut commands: Commands, query: Query<(&Transform, Entity), With<Rock>>) {
    for (transform, entity) in query.iter() {
        if transform.translation.x < ROCK_MIN_X {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn spawn_rock(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let mut entity = commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_xyz(WIDTH / 2.0 + 60.0, 0.0, 1.0),
        texture: asset_server.load("rock.png"),
        ..default()
    });

    entity.insert(HorizontalVelocity(250.0)).insert(Rock);

    add_collision_polygon(&mut entity, ROCK_UP_POINTS.to_vec());
}

fn add_collision_polygon(entity: &mut EntityCommands, coords: Vec<(f32, f32)>) {
    let fill_color = Color::rgba(0.2, 0.2, 0.8, 0.6);

    let points = coords.iter().map(to_point2).collect();
    let polygon = ConvexPolygon::try_new(points).unwrap();
    entity.insert(CollisionPolygon { polygon });

    let vecs: Vec<Vec2> = coords.iter().map(to_vec2).collect();

    let polygon = shapes::Polygon {
        points: vecs,
        closed: true,
    };

    let child = entity
        .commands()
        .spawn_bundle(GeometryBuilder::build_as(
            &polygon,
            DrawMode::Fill(FillMode::color(fill_color)),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ))
        .id();

    entity.push_children(&[child]);
}
