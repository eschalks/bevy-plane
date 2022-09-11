mod rocks;
mod text;

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use ncollide2d::na::Vector2;
use ncollide2d::shape::Cuboid;
use rocks::*;
use text::*;

pub type PlayerShape = Cuboid<f32>;

#[derive(Component, Debug)]
struct Background {
    width: f32,
}

#[derive(Component)]
pub struct Player {
    velocity: f32,
    shape: PlayerShape,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Start,
    Playing,
    Paused,
    GameOver,
}
struct GameSpeed(f32);

#[derive(Component)]
struct RemoveAfterState;

#[derive(Component)]
struct HorizontalVelocity(f32);

pub struct Score(u64); // Clearly this needs to be u64 in case someone ever scores over 4 billion

#[derive(Component)]
struct ScoreText;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 480.0;
const GRAVITY: f32 = 450.0;
const BUMP: f32 = GRAVITY * 0.65;
const PLAYER_WIDTH: f32 = 88.0;
const PLAYER_HEIGHT: f32 = 73.0;

const GROUND_WIDTH: f32 = 808.0;
const GROUND_HEIGHT: f32 = 73.0;

// At this velocity, the player is facing downwards
const FREE_FALL_VELOCITY: f32 = BUMP - GRAVITY * 1.6;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            ..default()
        })
        .insert_resource(GameSpeed(1.0))
        .insert_resource(RockTimer(Timer::from_seconds(0.0, false)))
        .insert_resource(Score(0))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_state(GameState::Start)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_enter(GameState::Start).with_system(setup_start))
        .add_system_set(SystemSet::on_update(GameState::Start).with_system(wait_for_click))
        .add_system_set(SystemSet::on_exit(GameState::Start).with_system(state_cleanup_system))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(rock_spawn_system)
                .with_system(loop_background)
                .with_system(horizontal_movement)
                .with_system(player_system)
                .with_system(rock_system)
                .with_system(collision_system),
        )
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(setup_game_over))
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(wait_for_click))
        .add_system_set(
            SystemSet::on_exit(GameState::GameOver)
                .with_system(reset_game)
                .with_system(state_cleanup_system),
        )
        .add_system(score_text_system)
        .add_system(bitmap_font_system)
        .run()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    spawn_background(
        &mut commands,
        asset_server.load("background.png"),
        0.0,
        0.0,
        0.0,
        WIDTH,
        150.0,
        false,
    );
    spawn_background(
        &mut commands,
        asset_server.load("groundGrass.png"),
        0.0,
        -HEIGHT / 2.0 + GROUND_HEIGHT / 2.0 - 1.0,
        3.0,
        GROUND_WIDTH,
        300.0,
        false,
    );
    spawn_background(
        &mut commands,
        asset_server.load("groundDirt.png"),
        -132.0,
        HEIGHT / 2.0 - GROUND_HEIGHT / 2.0 + 1.0,
        3.0,
        GROUND_WIDTH,
        300.0,
        true,
    );

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("Planes/planeBlue1.png"),
            transform: Transform::from_xyz(-200.0, 0.0, 1.0).with_scale(Vec3::new(0.5, 0.5, 1.0)),
            ..default()
        })
        .insert(Player {
            velocity: BUMP,
            shape: Cuboid::new(Vector2::new(PLAYER_WIDTH / 4.0, PLAYER_HEIGHT / 4.0)),
        });

    commands
        .spawn_bundle(
            BitmapTextBundle::new(WIDTH / 2.0 - 15.0, HEIGHT / 2.0 - 75.0)
                .with_anchor(TextAnchor::Right)
        )
        .insert(ScoreText);

    commands.insert_resource(create_bitmap_font(asset_server));
}

fn setup_start(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("UI/textGetReady.png"),
            transform: Transform::from_xyz(0.0, 100.0, 5.0),
            ..default()
        })
        .insert(RemoveAfterState);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("UI/tapLeft.png"),
            transform: Transform::from_xyz(-200.0 + PLAYER_WIDTH / 1.5, 0.0, 1.0)
                .with_scale(Vec3::new(0.5, 0.5, 1.0)),
            ..default()
        })
        .insert(RemoveAfterState);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("UI/tapRight.png"),
            transform: Transform::from_xyz(-200.0 - PLAYER_WIDTH / 1.5, 0.0, 1.0)
                .with_scale(Vec3::new(0.5, 0.5, 1.0)),
            ..default()
        })
        .insert(RemoveAfterState);
}

fn setup_game_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("UI/textGameOver.png"),
            transform: Transform::from_xyz(0.0, 100.0, 5.0),
            ..default()
        })
        .insert(RemoveAfterState);
}

fn score_text_system(score: Res<Score>, mut text_query: Query<&mut BitmapText, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }

    let mut text = text_query.single_mut();
    text.text = score.0.to_string();
}

fn wait_for_click(mut buttons: ResMut<Input<MouseButton>>, mut state: ResMut<State<GameState>>) {
    if buttons.just_pressed(MouseButton::Left) {
        let next_state = match state.current() {
            GameState::GameOver => GameState::Start,
            _ => GameState::Playing,
        };

        state.set(next_state).unwrap();
        buttons.reset(MouseButton::Left);
    }
}

fn spawn_background(
    commands: &mut Commands,
    texture: Handle<Image>,
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    velocity: f32,
    flip_y: bool,
) {
    for i in 0..2 {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    flip_y,
                    ..default()
                },
                texture: texture.clone(),
                transform: Transform::from_xyz(i as f32 * width + x, y, z),
                ..default()
            })
            .insert(Background { width })
            .insert(HorizontalVelocity(velocity));
    }
}

fn loop_background(mut query: Query<(&mut Transform, &Background)>) {
    for (mut t, background) in query.iter_mut() {
        if t.translation.x < -background.width {
            t.translation.x += background.width * 2.0;
        }
    }
}

fn horizontal_movement(
    mut query: Query<(&mut Transform, &HorizontalVelocity)>,
    time: Res<Time>,
    speed: Res<GameSpeed>,
) {
    let dt = time.delta_seconds();
    let speed = speed.0;

    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x -= dt * speed * velocity.0;
    }
}

fn player_system(
    mut query: Query<(&mut Player, &mut Transform)>,
    buttons: Res<Input<MouseButton>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let (mut player, mut transform) = query.single_mut();

    if buttons.just_pressed(MouseButton::Left) {
        player.velocity = BUMP;
    }

    let angle = if player.velocity >= 0.0 {
        (player.velocity / BUMP) * (PI / 6.0)
    } else if player.velocity > FREE_FALL_VELOCITY {
        (PI * 2.0) - (player.velocity / FREE_FALL_VELOCITY) * (PI / 2.0)
    } else {
        PI * 1.5
    };

    transform.rotation = Quat::from_rotation_z(angle);

    transform.translation.y += player.velocity * dt;
    player.velocity -= GRAVITY * dt;
}

fn reset_game(
    mut commands: Commands,
    mut rock_timer: ResMut<RockTimer>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    rocks: Query<Entity, With<Rock>>,
    mut score: ResMut<Score>,
) {
    rock_timer.0.reset();

    let (mut player_transform, mut player) = player_query.single_mut();
    player_transform.translation.y = 0.0;
    player_transform.rotation = Quat::IDENTITY;
    player.velocity = BUMP;

    for rock in rocks.iter() {
        commands.entity(rock).despawn_recursive();
    }

    score.0 = 0;
}

fn state_cleanup_system(mut commands: Commands, entities: Query<Entity, With<RemoveAfterState>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
