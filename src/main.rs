use std::fmt::Display;
use bevy::prelude::*;
use bevy::window::ExitCondition;
use rand::Rng;

#[derive(Default, Clone, Eq, PartialEq, Hash)]
enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
    GameWin,
}

#[derive(Event)]
struct DespawnEvent;

#[derive(Component)]
struct Player; // Represents the player entity

#[derive(Component)]
struct Block;

#[derive(Component)]
#[require(Velocity)]
struct Ball;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Score(u32); // Represents the player's score

#[derive(Component)]
struct DespawnOnGameOver;
                   
#[derive(Component)]
struct PauseText;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct GameWinText;

#[derive(Resource, Default)]
struct State(GameState); // Holds the current game state

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Constants for the window size and player size
const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 700.0;
const PLAYER_SIZE: f32 = 200.0;
const PLAYER_WIDTH: f32 = 15.0; // Thickness of the player paddle
const BLOCK_HEIGHT: f32 = WINDOW_HEIGHT / 20.0; // Height of each blocks
const BLOCK_WIDTH: f32 = WINDOW_WIDTH / 6.0; // Width of each block
const BALL_SIZE: f32 = 20.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Rust Breakout"), 
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }), // Set the window title and size
            exit_condition: ExitCondition::OnPrimaryClosed,
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.4, 0.4, 0.4))) // Set the background color
        .insert_resource(State(GameState::Playing)) // Initialize the game state
        .add_event::<DespawnEvent>() // Add a custom event for despawning entities
        .add_systems(Startup, (spawn_camera,
                               spawn_map,
                               spawn_blocks)) // Startup runs once on launch
        .add_systems(Update, (player_movement,
                              ball_movement,
                              ball_collision,
                              block_collision,
                              state_handler, // Handle game state changes
                              despawn_handler, // Handle despawning entities
                              pause_game,
                              game_win,
                              game_over)) // Update runs every frame
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default()); // Spawn a 2D camera
}

fn spawn_map(mut commands: Commands,
             mut mesh_assets: ResMut<Assets<Mesh>>,
             mut material_assets: ResMut<Assets<ColorMaterial>>) {

    // Create a rectangle mesh to represent the player
    let player_mesh = mesh_assets.add(Rectangle::new(PLAYER_SIZE, PLAYER_WIDTH));
    let player_material = material_assets.add(Color::srgb(1.0, 0.0, 0.0));

    // Create a ball that bounces between player and blocks
    let ball_mesh = mesh_assets.add(Circle::new(BALL_SIZE));
    let ball_material = material_assets.add(Color::srgb(0.0, 1.0, 0.0));

    // Spawn the player at the bottom of the window
    commands.spawn((
        Player,
        DespawnOnGameOver, // This component will be used to despawn the player on game over
        Transform::from_xyz(0.0, WINDOW_HEIGHT / -2.0 + 50.0, 0.0), 
        Mesh2d(player_mesh),
        MeshMaterial2d(player_material),
    ));

    // Spawn the ball at the center of the window with an initial downward velocity
    commands.spawn((
        Ball,
        DespawnOnGameOver, // This component will be used to despawn the ball on game over
        Transform::from_xyz(0.0, 0.0, 0.0), // Center of the window
        Velocity(Vec2::new(0.0, -400.0)), // Initial velocity
        Mesh2d(ball_mesh),
        MeshMaterial2d(ball_material),
    ));

    // Spawn the score text in the top right corner
    commands.spawn((
        Score(0),
        DespawnOnGameOver, // This component will be used to despawn the score text on game over
        Text2d::new("Score: 0"),
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - 100.0, WINDOW_HEIGHT / -2.0 + 25.0, 0.0),
        TextFont {
            font_size: 20.0,
            ..default()
        },
    ));
}

fn player_movement(mut pos: Query<&mut Transform, With<Player>>,
                   state: Res<State>,
                   keyboard_input: Res<ButtonInput<KeyCode>>) {

    let playing = state.0 == GameState::Playing; // Check if the game is in playing state

    for mut transform in pos.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyA)
            && playing
            && transform.translation.x > WINDOW_WIDTH / -2.0 + PLAYER_SIZE * 0.75 {
            transform.translation.x -= 5.0; // Move left
        }
        if keyboard_input.pressed(KeyCode::KeyD)
            && playing
            && transform.translation.x < WINDOW_WIDTH / 2.0 - PLAYER_SIZE * 0.75 {
            transform.translation.x += 5.0; // Move right
        }
    }
}

fn ball_movement(mut ball: Query<(&mut Transform, &mut Velocity), With<Ball>>,
                 time: Res<Time>,
                 state: Res<State>,){

    let playing = state.0 == GameState::Playing;

    for (mut transform, mut vel) in ball.iter_mut() {
        // Update position
        if playing {
            // Only update position if the game is not paused
            transform.translation.x += vel.0.x * time.delta_secs();
            transform.translation.y += vel.0.y * time.delta_secs();
        }

        // Bounce off walls
        if transform.translation.x < -WINDOW_WIDTH / 2.0 + BALL_SIZE / 2.0 ||
           transform.translation.x > WINDOW_WIDTH / 2.0 - BALL_SIZE / 2.0 {
            vel.0.x = -vel.0.x; // Invert the x velocity
        }
        if transform.translation.y > WINDOW_HEIGHT / 2.0 - BALL_SIZE / 2.0 {
            vel.0.y = -vel.0.y; // Invert the y velocity
        }
    }
}

fn ball_collision(mut balls: Query<(&Transform, &mut Velocity), With<Ball>>,
                  player: Query<&Transform, With<Player>>) {

    if let Ok(player_tf) = player.single() {

        for (ball_tf, mut vel) in balls.iter_mut() {

            if ball_tf.translation.y <= player_tf.translation.y + BALL_SIZE / 2.0 + PLAYER_WIDTH / 2.0
                && ball_tf.translation.y >= player_tf.translation.y - PLAYER_WIDTH / 2.0
                && ball_tf.translation.x >= player_tf.translation.x - PLAYER_SIZE / 2.0
                && ball_tf.translation.x <= player_tf.translation.x + PLAYER_SIZE / 2.0 {

                vel.0.y = -vel.0.y;

                //TODO: Adjust horizontal velocity based on where the ball hits the paddle
                let mut rng = rand::thread_rng();
                vel.0.x = rng.gen_range(-150.0..=150.0);
            }
        }
    }
}

// End game if ball hits bottom of screen
fn game_over(mut commands: Commands,
             score: Query<&Score>,
             mut state: ResMut<State>,
             transform: Query<&Transform, With<Ball>>) {

        for ball_tf in transform.iter() {
        if ball_tf.translation.y < -WINDOW_HEIGHT / 2.0 + BALL_SIZE / 2.0 {

            state.0 = GameState::GameOver; // Set game state to GameOver
           if let Ok(score) = score.single() {
                commands.spawn((
                    Text2d::new(format!("Game Over!\nYour Score: {}", score.0)),
                    TextFont {
                        font_size: 50.0,
                        ..default()
                    },
                ));
            } 
        }
    }
}

fn pause_game(mut time: ResMut<Time<Virtual>>,
              mut commands: Commands,
              mut state: ResMut<State>,
              text: Query<Entity, With<PauseText>>,
              keyboard_input: Res<ButtonInput<KeyCode>>) {

    if keyboard_input.just_pressed(KeyCode::Space) {
        if state.0 == GameState::Paused {
            state.0 = GameState::Playing; // Set game state to Playing
            time.unpause(); 
            for entity in text.iter() {
                commands.entity(entity).despawn(); // Remove pause text
            }
        } else if state.0 == GameState::Playing {
            state.0 = GameState::Paused; // Set game state to Paused
            time.pause();
            commands.spawn((
                PauseText,
                Text2d::new("Paused"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
            ));
        }
    }
}

fn spawn_blocks(mut commands: Commands,
                mut mesh_assets: ResMut<Assets<Mesh>>,
                mut material_assets: ResMut<Assets<ColorMaterial>>) {

    let block_mesh = mesh_assets.add(Rectangle::new(BLOCK_WIDTH, BLOCK_HEIGHT));
    let block_material = material_assets.add(Color::srgb(0.0, 0.4, 1.0));

    for i in 0..5 {
        for j in 0..5 {
            commands.spawn((
                Block,
                DespawnOnGameOver, // This component will be used to despawn blocks on game over
                Transform::from_xyz(
                    (i as f32 - 2.0) * (BLOCK_WIDTH + 15.0), // Position blocks in a grid
                    (j as f32 + 3.0) * (BLOCK_HEIGHT + 10.0),
                    0.0,
                ),
                Mesh2d(block_mesh.clone()),
                MeshMaterial2d(block_material.clone()),
            ));
        }
    }
}

fn block_collision(mut blocks: Query<(Entity, &Transform), With<Block>>,
                   mut ball: Query<(&Transform, &mut Velocity), With<Ball>>,
                   mut score: Query<(&mut Score, &mut Text2d), With<Score>>,
                   mut commands: Commands) {

    //TODO: Optimize block collision detection
    for (ball_tf, mut vel) in ball.iter_mut() {
        for (block_entity, block_tf) in blocks.iter_mut() {
            if ball_tf.translation.x + BALL_SIZE / 2.0 >= block_tf.translation.x - BLOCK_WIDTH / 2.0 &&
               ball_tf.translation.x - BALL_SIZE / 2.0 <= block_tf.translation.x + BLOCK_WIDTH / 2.0 &&
               ball_tf.translation.y + BALL_SIZE / 2.0 >= block_tf.translation.y - BLOCK_HEIGHT / 2.0 &&
               ball_tf.translation.y - BALL_SIZE / 2.0 <= block_tf.translation.y + BLOCK_HEIGHT / 2.0 {

                vel.0.y = -vel.0.y; // Bounce the ball off the block

                let mut rng = rand::thread_rng();
                vel.0.x = rng.gen_range(-150.0..=150.0);

                commands.entity(block_entity).despawn(); // Remove the block
                if let Ok((mut score, mut text)) = score.single_mut() {
                    score.0 += 1; // Increment the score
                    let length = text.len();
                    text.replace_range(0..length, format!("Score: {}", score.0).as_str()); // Update the score text
                }
            }
        }
    }
}

fn game_win(blocks: Query<&Block>,
            mut commands: Commands,
            mut time: ResMut<Time<Virtual>>,
            mut state: ResMut<State>) {

    if blocks.is_empty() && state.0 == GameState::Playing {
        state.0 = GameState::GameWin; // Set game state to GameWin
        time.pause(); // Pause the game when all blocks are destroyed
        commands.spawn((
            GameWinText,
            Text2d::new(format!("You Win!")),
            TextFont {
                font_size: 50.0,
                ..default()
            },
        ));
    }
}

//TODO: Implement a system to handle game state changes
fn state_handler(state: Res<State>,
                 keyboard_input: Res<ButtonInput<KeyCode>>,
                 mut event_writer: EventWriter<DespawnEvent>) {

    match state.0 {
        GameState::GameOver => {
            event_writer.write(DespawnEvent); // Trigger despawn event for game over
            if keyboard_input.just_pressed(KeyCode::Escape) {
                std::process::exit(0);
            }
        }
        GameState::GameWin => {
            event_writer.write(DespawnEvent); // Trigger despawn event for game over
            if keyboard_input.just_pressed(KeyCode::Escape) {
                std::process::exit(0);
            }
        }
        GameState::Paused => {
            // No action needed for paused state
        }
        _ => {}
    }
}

fn despawn_handler(mut reader: EventReader<DespawnEvent>,
                   entities: Query<Entity, With<DespawnOnGameOver>>,
                   mut commands: Commands) {

    for _ in reader.read(){
       for entity in entities.iter() {
            commands.entity(entity).despawn(); // Despawn all entities with the DespawnOnGameOver component
        } 
    }
}
