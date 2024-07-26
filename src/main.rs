use std::time::Duration;

use bevy::prelude::*;

const BIRD_SCALE: f32 = 3.0; // Adjust this value to change the bird's size

#[derive(Component)]
struct Background;

#[derive(Component, Debug)]
pub struct AnimationTimer(Timer);

#[derive(Component, Debug)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Resource)]
pub struct GameState {
    pub is_game_over: bool,
}

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Event)]
pub struct RestartEvent;

#[derive(Component, Debug)]
pub struct GameCamera;

#[derive(Component, Debug)]
pub struct Ground;

#[derive(Component, Debug)]
pub struct GameOverText;

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
                value: Vec2::new(2., 0.),
            },
            gravity: Gravity {},
            player: Player {},
        }
    }
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_level, spawn_player).chain())
            .add_systems(
                Update,
                (
                    update_velocity_on_space,
                    handle_game_over,
                    check_for_restart,
                    animate_sprite,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_position,
                    apply_gravity,
                    check_collision,
                    camera_follow_player,
                )
                    .chain(),
            )
            .add_systems(Update, handle_restart_event)
            .insert_resource(GameState {
                is_game_over: false,
            })
            .add_event::<GameOverEvent>()
            .add_event::<RestartEvent>();
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (800.0, 600.0).into(),
                    title: "Flappy Circle".to_string(),
                    ..default()
                }),
                ..default()
            }),
            GamePlugin,
        ))
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(16)))
        .run();
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("textures/Bird1-1.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 4, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 3 };
    commands.spawn((
        SpriteBundle {
            texture: texture_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0), // Position the player at the center
                scale: Vec3::splat(BIRD_SCALE),
                ..Default::default()
            },
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: animation_indices.first,
        },
        PlayerBundle::default(),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
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

fn spawn_ground(commands: &mut Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(10000.0, 300.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -400.0, 0.0),
            ..default()
        },
        Ground,
    ));
}

fn spawn_camera(commands: &mut Commands) {
    commands.spawn((Camera2dBundle::default(), GameCamera));
}

fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_camera(&mut commands);
    setup_background(&mut commands, asset_server);
    spawn_ground(&mut commands);
}

fn camera_follow_player(
    q_target: Query<Entity, With<Player>>,
    mut transform_params: ParamSet<(TransformHelper, Query<&mut Transform, With<GameCamera>>)>,
) {
    if let Ok(e_target) = q_target.get_single() {
        // compute its actual current GlobalTransform
        // (could be Err if entity doesn't have transforms)
        let Ok(global) = transform_params.p0().compute_global_transform(e_target) else {
            return;
        };
        // get camera transform and make it look at the global translation
        transform_params.p1().single_mut().translation.x = global.translation().x;
    }
}

fn update_velocity_on_space(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut TextureAtlas), With<Player>>,
    game_state: Res<GameState>,
) {
    if !game_state.is_game_over && keyboard_input.just_pressed(KeyCode::Space) {
        for (mut velocity, mut sprite) in query.iter_mut() {
            velocity.value.y = 5.0;
            sprite.index = 0; // Reset animation to first frame when jumping
        }
    }
}

fn check_collision(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<Player>>,
    ground_query: Query<&Transform, With<Ground>>,
    mut game_over_events: EventWriter<GameOverEvent>,
    game_state: Res<GameState>,
) {
    if game_state.is_game_over {
        return;
    }

    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };
    let Ok(ground_transform) = ground_query.get_single() else {
        return;
    };

    let player_y = player_transform.translation.y - (8.0 * BIRD_SCALE);
    let ground_y = ground_transform.translation.y + 150.0;

    if player_y <= ground_y {
        game_over_events.send(GameOverEvent);
        commands.entity(player_entity).despawn();
    }
}

fn handle_game_over(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut game_over_events: EventReader<GameOverEvent>,
    asset_server: Res<AssetServer>,
) {
    for _ in game_over_events.read() {
        if !game_state.is_game_over {
            game_state.is_game_over = true;

            commands.spawn((
                TextBundle::from_section(
                    "Game over!\nPress R to restart",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    left: Val::Px(400.0),
                    ..default()
                }),
                GameOverText,
            ));
        }
    }
}

fn check_for_restart(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_state: Res<GameState>,
    mut restart_events: EventWriter<RestartEvent>,
) {
    if game_state.is_game_over && keyboard_input.just_pressed(KeyCode::KeyR) {
        restart_events.send(RestartEvent);
    }
}

fn handle_restart_event(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut restart_events: EventReader<RestartEvent>,
    player_query: Query<Entity, With<Player>>,
    ground_query: Query<Entity, With<Ground>>,
    game_over_text_query: Query<Entity, With<GameOverText>>,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !restart_events.is_empty() {
        // Reset game state
        game_state.is_game_over = false;

        // Despawn existing entities
        for entity in player_query
            .iter()
            .chain(ground_query.iter())
            .chain(game_over_text_query.iter())
        {
            commands.entity(entity).despawn();
        }

        // Respawn player and ground
        spawn_ground(&mut commands);
        spawn_player(commands, asset_server, texture_atlas_layouts);
        restart_events.clear();
    }
}

fn setup_background(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let background_image = asset_server.load("textures/Background5.png");
    commands.spawn((
        SpriteBundle {
            texture: background_image,
            transform: Transform {
                // The scale might need adjusting depending on your image size and desired coverage
                scale: Vec3::new(1.0, 1.0, 1.0),
                translation: Vec3::new(200.0, 300.0, -1.0),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(Vec2::new(1200.0, 1200.0)), // Match this to your window size
                ..default()
            },
            ..default()
        },
        Background,
    ));
}
