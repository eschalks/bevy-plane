mod rocks;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use ncollide2d::na::Vector2;
use ncollide2d::shape::Cuboid;
use rocks::*;

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
struct HorizontalVelocity(f32);

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 480.0;
const GRAVITY: f32 = 150.0;
const BUMP: f32 = GRAVITY * 1.1;
const PLAYER_WIDTH: f32 = 88.0;
const PLAYER_HEIGHT: f32 = 73.0;

const GROUND_WIDTH: f32 = 808.0;
const GROUND_HEIGHT: f32 = 73.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            ..default()
        })
        .insert_resource(GameSpeed(1.0))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_state(GameState::Start)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_update(GameState::Start).with_system(wait_for_click))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(loop_background)
                .with_system(horizontal_movement)
                .with_system(player_system)
                .with_system(rock_system)
                .with_system(collision_system),
        )
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(wait_for_click))
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(reset_game))
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
        -HEIGHT / 2.0 + GROUND_HEIGHT / 2.0,
        3.0,
        GROUND_WIDTH,
        300.0,
        false,
    );
    spawn_background(
        &mut commands,
        asset_server.load("groundDirt.png"),
        -132.0,
        HEIGHT / 2.0 - GROUND_HEIGHT / 2.0,
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

    spawn_rock(&mut commands, asset_server);
}

fn wait_for_click(buttons: Res<Input<MouseButton>>, mut state: ResMut<State<GameState>>) {
    if buttons.just_pressed(MouseButton::Left) {
        state.set(GameState::Playing).unwrap();
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

fn loop_background(mut query: Query<&mut Transform, With<Background>>) {
    for mut t in query.iter_mut() {
        if t.translation.x < -WIDTH {
            t.translation.x += WIDTH * 2.0;
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
    for (mut player, mut transform) in query.iter_mut() {
        if buttons.just_pressed(MouseButton::Left) {
            player.velocity = BUMP;
            transform.rotation = Quat::from_rotation_z((30.0_f32).to_radians());
        }

        let (_, angle) = transform.rotation.to_axis_angle();
        let mut new_angle = (angle.to_degrees() - time.delta_seconds() * 30.0);
        if new_angle < 0.0 {
            new_angle += 360.0;
        }

        transform.rotation = Quat::from_rotation_z(new_angle.to_radians());

        transform.translation.y += player.velocity * dt;
        player.velocity -= GRAVITY * dt;
    }
}

fn reset_game() {}
