use std::time::Duration;

use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct GameCamera;

#[derive(Component, Debug)]
pub struct Velocity {
    pub value: Vec2,
}

#[derive(Component, Debug)]
pub struct Gravity;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Bundle, Debug)]
pub struct PlayerBundle {
    pub velocity: Velocity,
    pub gravity: Gravity,
    pub player: Player,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            velocity: Velocity {
                value: Vec2::new(0., 0.),
            },
            gravity: Gravity {},
            player: Player {},
        }
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(Update, update_velocity_on_space);
        app.add_systems(FixedUpdate, camera_follow_player);
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
              resolution: (800.0, 600.0).into(),
              title: "Flappy Circle".to_string(),
              ..default()
            }),
            ..default()
          }), PlayerPlugin))
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(16)))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.808, 0.922)))
        .add_systems(Startup, setup_level)
        .add_systems(FixedUpdate, (update_position, apply_gravity).chain())
        .run();
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle = asset_server.load("circle.png");

    commands.spawn((
        SpriteBundle {
            texture: texture_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0), // Position the player at the center
                ..Default::default()
            },
            ..Default::default()
        },
        PlayerBundle::default(),
    ));
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.value.x;
        transform.translation.y += velocity.value.y;
    }
}

fn apply_gravity(mut query: Query<&mut Velocity, With<Gravity>>) {
    for mut velocity in query.iter_mut() {
        velocity.value.y += -0.1;
    }
}

fn setup_level(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), GameCamera));
}

fn camera_follow_player(
    q_target: Query<Entity, With<Player>>,
    mut transform_params: ParamSet<(
        TransformHelper,
        Query<&mut Transform, With<GameCamera>>,
    )>
) {
    if let Ok(e_target ) = q_target.get_single() {
        // compute its actual current GlobalTransform
        // (could be Err if entity doesn't have transforms)
        let Ok(global) = transform_params.p0().compute_global_transform(e_target) else {
            return;
        };
        // get camera transform and make it look at the global translation
        transform_params.p1().single_mut().translation = global.translation();
    }
}

fn update_velocity_on_space(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut velocity in query.iter_mut() {
            velocity.value.y = 5.0;
        }
    }
}