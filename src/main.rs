use std::fmt::Display;
use bevy::prelude::*;
use bevy::window::ExitCondition;
use rand::Rng;

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
        .add_systems(Startup, (spawn_camera,
                               spawn_map,
                               spawn_blocks)) // Startup runs once on launch
        .add_systems(Update, (player_movement,
                              ball_movement,
                              ball_collision,
                              block_collision,
                              pause_game,
                              game_win,
                              game_over.after(game_win))) // Update runs every frame
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
        Transform::from_xyz(0.0, WINDOW_HEIGHT / -2.0 + 50.0, 0.0), 
        Mesh2d(player_mesh),
        MeshMaterial2d(player_material),
    ));

    // Spawn the ball at the center of the window with an initial downward velocity
    commands.spawn((
        Ball,
        Transform::from_xyz(0.0, 0.0, 0.0), // Center of the window
        Velocity(Vec2::new(0.0, -400.0)), // Initial velocity
        Mesh2d(ball_mesh),
        MeshMaterial2d(ball_material),
    ));

    // Spawn the score text in the top right corner
    commands.spawn((
        Score(0),
        Text2d::new("Score: 0"),
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - 100.0, WINDOW_HEIGHT / -2.0 + 25.0, 0.0),
        TextFont {
            font_size: 20.0,
            ..default()
        },
    ));
}

fn player_movement(mut pos: Query<&mut Transform, With<Player>>,
                   time: Res<Time<Virtual>>,
                   keyboard_input: Res<ButtonInput<KeyCode>>) {

    for mut transform in pos.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyA)
            && !time.is_paused()
            && transform.translation.x > WINDOW_WIDTH / -2.0 + PLAYER_SIZE * 0.75 {
            transform.translation.x -= 5.0; // Move left
        }
        if keyboard_input.pressed(KeyCode::KeyD)
            && !time.is_paused()
            && transform.translation.x < WINDOW_WIDTH / 2.0 - PLAYER_SIZE * 0.75 {
            transform.translation.x += 5.0; // Move right
        }
    }
}

fn ball_movement(mut ball: Query<(&mut Transform, &mut Velocity), With<Ball>>,
                 time: Res<Time>) {

    for (mut transform, mut vel) in ball.iter_mut() {
        // Update position
        transform.translation.x += vel.0.x * time.delta_secs();
        transform.translation.y += vel.0.y * time.delta_secs();

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
                let mut rng = rand::rng();
                vel.0.x = rng.random_range(-150.0..=150.0);
            }
        }
    }
}

// End game if ball hits bottom of screen
fn game_over(mut time: ResMut<Time<Virtual>>,
             mut commands: Commands,
             text: Query<Entity, With<Text2d>>,
             ball_entity: Query<Entity, With<Ball>>, 
             player_entity: Query<Entity, With<Player>>, 
             block_entity: Query<Entity, With<Block>>,
             transform: Query<&Transform, With<Ball>>) {

        for ball_tf in transform.iter() {
        if ball_tf.translation.y < -WINDOW_HEIGHT / 2.0 + BALL_SIZE / 2.0 {
            
            for entity in text.iter() {
                commands.entity(entity).despawn(); //Remove existing UI text
            }

            time.pause();
            commands.spawn((
                Text2d::new("Game Over!"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
            ));
            for entity in ball_entity.iter() {
                commands.entity(entity).despawn(); // Remove ball entity
            }
            for entity in player_entity.iter() {
                commands.entity(entity).despawn(); // Remove player entity
            }
            for entity in block_entity.iter() {
                commands.entity(entity).despawn(); // Remove block entities
            }
        }
    }
}

fn pause_game(mut time: ResMut<Time<Virtual>>,
              mut commands: Commands,
              text: Query<Entity, With<Text2d>>,
              keyboard_input: Res<ButtonInput<KeyCode>>) {

    //TODO: Bug fix - score text disappears when paused
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if time.is_paused() {
            time.unpause();
            for entity in text.iter() {
                commands.entity(entity).despawn(); // Remove pause text
            }
        } else {
            time.pause();
            commands.spawn((
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

    let block_mesh = mesh_assets.add(Rectangle::new(WINDOW_WIDTH / 6.0, WINDOW_HEIGHT / 20.0));
    let block_material = material_assets.add(Color::srgb(0.0, 0.4, 1.0));

    for i in 0..5 {
        for j in 0..5 {
            commands.spawn((
                Block,
                Transform::from_xyz(
                    (i as f32 - 2.0) * (WINDOW_WIDTH / 6.0 + 15.0), // Position blocks in a grid
                    (j as f32 + 3.0) * (WINDOW_HEIGHT / 20.0 + 10.0),
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
            if ball_tf.translation.x >= block_tf.translation.x - WINDOW_WIDTH / 12.0 &&
               ball_tf.translation.x <= block_tf.translation.x + WINDOW_WIDTH / 12.0 &&
               ball_tf.translation.y >= block_tf.translation.y - WINDOW_HEIGHT / 40.0 &&
               ball_tf.translation.y <= block_tf.translation.y + WINDOW_HEIGHT / 40.0 {

                vel.0.y = -vel.0.y; // Bounce the ball off the block
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
            score: Query<(&Score, Entity), With<Score>>,
            ball: Query<Entity, With<Ball>>,
            player: Query<Entity, With<Player>>) {

    if blocks.is_empty() {
        time.pause(); // Pause the game when all blocks are destroyed
        if let Ok(score) = score.single() {
            commands.spawn((
                Text2d::new(format!("You Win!\nScore: {}", score.0)),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
            ));
        }
       // Despawn player, ball, and score text
        for entity in ball.iter() {
            commands.entity(entity).despawn();
        }
        for entity in player.iter() {
            commands.entity(entity).despawn();
        }
        for (_, entity) in score.iter() {
            commands.entity(entity).despawn();
        } 
    }
}
