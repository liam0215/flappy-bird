use std::time::Duration;
use bevy::prelude::*;

const BIRD_SCALE: f32 = 1.0; // Adjust this value to change the bird's size
const GROUND_SCALE: f32 = 2.0; // Adjust this value to change the ground's size
const PIPE_SCALE: Vec3 = Vec3::new(3., 5., 1.); // Adjust this value to change the pipe's size

#[derive(Component, Debug)]
struct Pipe;

#[derive(Component, Debug)]
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
                    update_player_position,
                    apply_gravity,
                    check_collision,
                    camera_follow_player,
                    update_bg_position,
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
    let texture_handle = asset_server.load("textures/mooslisprites.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(180, 100), 2, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 1 };
    commands.spawn((
        SpriteBundle {
            texture: texture_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 2.0), // Position the player at the center
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
        AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
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

fn update_player_position(mut query: Query<(&Velocity, &mut Transform), With<Player>>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.value.x;
        transform.translation.y += velocity.value.y;
    }
}

fn update_bg_position(
    camera_query: Query<(&Transform, &GameCamera), Without<Background>>,
    mut bg_query: Query<&mut Transform, With<Background>>,
) {
    let (camera, _) = camera_query.single();
    let mut bg = bg_query.single_mut();

    bg.translation.x = camera.translation.x;
}

fn apply_gravity(mut query: Query<&mut Velocity, With<Gravity>>) {
    for mut velocity in query.iter_mut() {
        velocity.value.y += -0.1;
    }
}

fn spawn_ground(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let ground_image = asset_server.load("textures/ground.png");
    for i in 0..100 {
        commands.spawn((
            SpriteBundle {
                texture: ground_image.clone(),
                transform: Transform::from_xyz(64. * ((i as f32) - 5.), -268., 0.0)
                    .with_scale(Vec3::splat(GROUND_SCALE)),
                ..default()
            },
            Ground,
        ));
    }
}

fn spawn_camera(commands: &mut Commands) {
    commands.spawn((Camera2dBundle::default(), GameCamera));
}

fn setup_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    spawn_camera(&mut commands);
    setup_background(&mut commands, &asset_server);
    spawn_ground(&mut commands, &asset_server);
    spawn_pipe(&mut commands, &asset_server, &mut texture_atlas_layouts);
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
    pipe_query: Query<&Transform, With<Pipe>>,
    mut game_over_events: EventWriter<GameOverEvent>,
    game_state: Res<GameState>,
) {
    if game_state.is_game_over {
        return;
    }

    let (player_entity, player_transform) = player_query.single();
    let ground_transform = ground_query.iter().next().unwrap();

    let player_y = player_transform.translation.y - (50.0 * BIRD_SCALE);
    let ground_y = ground_transform.translation.y + (16.0 * GROUND_SCALE);

    if player_y <= ground_y {
        game_over_events.send(GameOverEvent);
        commands.entity(player_entity).despawn();
    } else {
        for pipe_transform in pipe_query.iter() {
            let pipe_x = pipe_transform.translation.x;
            let pipe_y = pipe_transform.translation.y;
            let player_x = player_transform.translation.x;
            let player_y = player_transform.translation.y;
            let pipe_half_w = (32. * PIPE_SCALE.x) / 2.0;
            let pipe_half_h = (48. * PIPE_SCALE.y) / 2.0;
            let player_half_w = - (90.0 * BIRD_SCALE) / 2.0;
            let player_half_h = (50.0 * BIRD_SCALE) / 2.0;
            if player_x + player_half_w >= pipe_x - pipe_half_w && player_x - player_half_w <= pipe_x + pipe_half_w {
                if player_y + player_half_h >= pipe_y - pipe_half_h && player_y - player_half_h <= pipe_y + pipe_half_h {
                    game_over_events.send(GameOverEvent);
                    commands.entity(player_entity).despawn();
                    break;
                }
            }
        }
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
    game_over_text_query: Query<Entity, With<GameOverText>>,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !restart_events.is_empty() {
        // Reset game state
        game_state.is_game_over = false;

        // Despawn existing entities
        for entity in player_query.iter().chain(game_over_text_query.iter()) {
            commands.entity(entity).despawn();
        }

        // Respawn player and ground
        spawn_player(commands, asset_server, texture_atlas_layouts);
        restart_events.clear();
    }
}

fn setup_background(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let background_image = asset_server.load("textures/Background5.png");
    commands.spawn((
        SpriteBundle {
            texture: background_image,
            transform: Transform {
                // The scale might need adjusting depending on your image size and desired coverage
                scale: Vec3::new(1.0, 1.0, 1.0),
                translation: Vec3::new(0., 100., -1.0),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(Vec2::new(756., 756.)), // Match this to your window size
                ..default()
            },
            ..default()
        },
        Background,
    ));
}

fn spawn_pipe(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("textures/PipeStyle5.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 48), 4, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    for i in 0..100 {
        let y = if rand::random() { -200. } else { 250. };
        commands.spawn((
            SpriteBundle {
                texture: texture_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(400. + (400. * (i as f32)), y, 1.0), // Position the player at the center
                    scale: PIPE_SCALE,
                    ..Default::default()
                },
                ..Default::default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
            Pipe,
        ));
    }
}
