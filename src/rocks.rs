use crate::{GameState, HorizontalVelocity, Player, PlayerShape, Score, HEIGHT, WIDTH};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::utils::Duration;
#[cfg(debug_assertions)]
use bevy_prototype_lyon::prelude::*;
use ncollide2d::na;
use ncollide2d::na::{Isometry2, Point2, Vector2};
use ncollide2d::query::{self, Proximity};
use ncollide2d::shape::ConvexPolygon;
use rand::prelude::*;

const ROCK_WIDTH: f32 = 108.0;
const ROCK_HEIGHT: f32 = 239.0;
const ROCK_MIN_X: f32 = -WIDTH / 2.0 - ROCK_WIDTH;

const ROCK_UP_POINTS: &'static [(f32, f32)] = &[
    (-ROCK_WIDTH / 2.0 + 6.0, -ROCK_HEIGHT / 2.0),
    (ROCK_WIDTH / 2.0 - 6.0, -ROCK_HEIGHT / 2.0),
    (12.0, ROCK_HEIGHT / 2.0),
];

const ROCK_DOWN_POINTS: &'static [(f32, f32)] = &[
    (12.0, -ROCK_HEIGHT / 2.0),
    (-ROCK_WIDTH / 2.0 + 6.0, ROCK_HEIGHT / 2.0),
    (ROCK_WIDTH / 2.0 - 6.0, ROCK_HEIGHT / 2.0),
];

pub struct RockTimer(pub Timer);

#[derive(Component)]
pub struct CollisionPolygon {
    polygon: ConvexPolygon<f32>,
}

#[derive(Component)]
pub struct Rock {
    has_scored: bool,
}

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
    let (player, player_transform) = player_query.single();

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
            return;
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

pub fn rock_system(
    mut commands: Commands,
    mut query: Query<(&Transform, Entity, &mut Rock)>,
    player_query: Query<&Transform, With<Player>>,
    mut score: ResMut<Score>,
) {
    let player_x = player_query.single().translation.x;

    for (transform, entity, mut rock) in query.iter_mut() {
        if transform.translation.x < ROCK_MIN_X {
            commands.entity(entity).despawn_recursive();
        }

        if !rock.has_scored && transform.translation.x < player_x {

            // If we fly inbetween two rocks it should still count as 1 point
            if !score.is_changed() {
                score.0 += 1;
            }

            rock.has_scored = true;
        }
    }
}

pub fn rock_spawn_system(
    mut commands: Commands,
    mut timer: ResMut<RockTimer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    if timer.0.tick(time.delta()).finished() {
        let mut rng = thread_rng();
        let scale = rng.gen_range(0.7..1.2);
        let rock_type = rng.gen_range(0..=2);
        spawn_rocks(&mut commands, asset_server, scale, rock_type);
        let next_time: f32 = rng.gen_range(0.4..1.5);
        timer.0.set_duration(Duration::from_secs_f32(next_time));
        timer.0.reset();
    }
}

fn spawn_rocks(commands: &mut Commands, asset_server: Res<AssetServer>, scale: f32, rock_type: u8) {
    let mut rock_descriptions: Vec<(f32, &str, Vec<(f32, f32)>)> = vec![];

    let scale = if rock_type == 2 { scale * 0.7 } else { scale };

    if rock_type != 0 {
        rock_descriptions.push((
            HEIGHT / -2.0 + (ROCK_HEIGHT * scale) / 2.0,
            "rockGrass.png",
            ROCK_UP_POINTS.to_vec(),
        ));
    }

    if rock_type != 1 {
        rock_descriptions.push((
            HEIGHT / 2.0 - (ROCK_HEIGHT * scale) / 2.0,
            "rockDown.png",
            ROCK_DOWN_POINTS.to_vec(),
        ));
    }

    for (y, texture, points) in rock_descriptions.iter() {
        let mut entity = commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(WIDTH / 2.0 + 60.0, *y, 1.0)
                .with_scale(Vec3::new(1.0, scale, 1.0)),
            texture: asset_server.load(*texture),
            ..default()
        });

        add_collision_polygon(&mut entity, points, scale);

        entity
            .insert(HorizontalVelocity(250.0))
            .insert(Rock { has_scored: false });
    }
}

fn add_collision_polygon(entity: &mut EntityCommands, coords: &Vec<(f32, f32)>, scale: f32) {
    let coords: Vec<(f32, f32)> = coords.iter().map(|(x, y)| (*x, y * scale)).collect();

    let points = coords.iter().map(to_point2).collect();
    let polygon = ConvexPolygon::try_new(points).unwrap();
    entity.insert(CollisionPolygon { polygon });

    // During debugging it's sometimes useful to be able to see the collision outline
    #[cfg(debug_assertions)]
    {
        let fill_color = Color::rgba(0.2, 0.2, 0.8, 0.6);

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
                Transform::from_xyz(0.0, 0.0, 2.0).with_scale(Vec3::new(1.0, 1.0 / scale, 1.0)),
            ))
            .id();

        entity.add_child(child);
    }
}
