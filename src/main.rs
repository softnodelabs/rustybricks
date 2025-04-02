use bevy::prelude::*;
use rand::prelude::*;

// Constants
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
const BALL_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const BALL_SPEED: f32 = 400.0;
const BRICK_SIZE: Vec2 = Vec2::new(80.0, 30.0);
const BRICK_ROWS: usize = 4;
const BRICK_COLUMNS: usize = 10;
const BRICK_SPACING: Vec2 = Vec2::new(10.0, 10.0);
const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);

// Components
#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball {
    velocity: Vec2,
}

#[derive(Component)]
struct Brick;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Collider {
    size: Vec2,
}

#[derive(Resource)]
struct GameState {
    score: u32,
    lives: u32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameState { score: 0, lives: 3 })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_paddle,
                move_ball,
                check_ball_collision,
                reset_ball,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
) {
    // Camera
    commands.spawn(Camera2d);

    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();

    // Paddle
    commands.spawn((
        Sprite {
            color: PADDLE_COLOR,
            custom_size: Some(PADDLE_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, -window_height / 2.0 + 50.0, 0.0),
        Paddle,
        Collider { size: PADDLE_SIZE },
    ));

    // Ball
    let mut rng = rand::thread_rng();
    let random_angle = rng.gen_range((-std::f32::consts::PI / 4.0)..(std::f32::consts::PI / 4.0));
    let direction = Vec2::new(random_angle.sin(), random_angle.cos()).normalize();

    commands.spawn((
        Sprite {
            color: BALL_COLOR,
            custom_size: Some(BALL_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, -window_height / 2.0 + 100.0, 0.0),
        Ball {
            velocity: direction * BALL_SPEED,
        },
        Collider { size: BALL_SIZE },
    ));

    // Bricks
    let total_width_of_bricks = (BRICK_SIZE.x + BRICK_SPACING.x) * BRICK_COLUMNS as f32 - BRICK_SPACING.x;
    let starting_x = -total_width_of_bricks / 2.0 + BRICK_SIZE.x / 2.0;
    let starting_y = window_height / 3.0;

    for row in 0..BRICK_ROWS {
        for column in 0..BRICK_COLUMNS {
            let brick_position = Vec2::new(
                starting_x + column as f32 * (BRICK_SIZE.x + BRICK_SPACING.x),
                starting_y - row as f32 * (BRICK_SIZE.y + BRICK_SPACING.y),
            );

            // Create a slightly different color for each row
            let row_factor = 1.0 - (row as f32 * 0.2);
            let brick_color = Color::srgb(0.2, 0.3, 0.8 * row_factor);

            commands.spawn((
                Sprite {
                    color: brick_color,
                    custom_size: Some(BRICK_SIZE),
                    ..default()
                },
                Transform::from_translation(brick_position.extend(0.0)),
                Brick,
                Collider { size: BRICK_SIZE },
            ));
        }
    }

    // Walls
    // Left wall
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(WALL_THICKNESS, window_height)),
            ..default()
        },
        Transform::from_xyz(-window_width / 2.0 - WALL_THICKNESS / 2.0, 0.0, 0.0),
        Wall,
        Collider {
            size: Vec2::new(WALL_THICKNESS, window_height),
        },
    ));

    // Right wall
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(WALL_THICKNESS, window_height)),
            ..default()
        },
        Transform::from_xyz(window_width / 2.0 + WALL_THICKNESS / 2.0, 0.0, 0.0),
        Wall,
        Collider {
            size: Vec2::new(WALL_THICKNESS, window_height),
        },
    ));

    // Top wall
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(window_width + WALL_THICKNESS * 2.0, WALL_THICKNESS)),
            ..default()
        },
        Transform::from_xyz(0.0, window_height / 2.0 + WALL_THICKNESS / 2.0, 0.0),
        Wall,
        Collider {
            size: Vec2::new(window_width + WALL_THICKNESS * 2.0, WALL_THICKNESS),
        },
    ));
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
    windows: Query<&Window>,
) {
    let mut paddle_transform = query.single_mut();
    let window = windows.single();
    let window_width = window.width();
    let half_paddle_width = PADDLE_SIZE.x / 2.0;
    let wall_width = WALL_THICKNESS;
    let boundary = window_width / 2.0 - half_paddle_width - wall_width;

    let paddle_speed = 500.0;
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    let mut position = paddle_transform.translation;
    position.x += direction * paddle_speed * time.delta_secs();
    position.x = position.x.clamp(-boundary, boundary);
    paddle_transform.translation = position;
}

fn move_ball(
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
    time: Res<Time>,
) {
    for (ball, mut transform) in ball_query.iter_mut() {
        transform.translation.x += ball.velocity.x * time.delta_secs();
        transform.translation.y += ball.velocity.y * time.delta_secs();
    }
}

// Simple AABB collision detection
fn check_collision(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    let a_min = Vec2::new(a_pos.x - a_size.x / 2.0, a_pos.y - a_size.y / 2.0);
    let a_max = Vec2::new(a_pos.x + a_size.x / 2.0, a_pos.y + a_size.y / 2.0);
    let b_min = Vec2::new(b_pos.x - b_size.x / 2.0, b_pos.y - b_size.y / 2.0);
    let b_max = Vec2::new(b_pos.x + b_size.x / 2.0, b_pos.y + b_size.y / 2.0);

    a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y
}

fn check_ball_collision(
    mut commands: Commands,
    mut ball_query: Query<(&mut Ball, &Transform, &Collider)>,
    collider_query: Query<(Entity, &Transform, &Collider, Option<&Brick>, Option<&Paddle>, Option<&Wall>)>,
    mut game_state: ResMut<GameState>,
) {
    for (mut ball, ball_transform, ball_collider) in ball_query.iter_mut() {
        let ball_position = ball_transform.translation.truncate();
        
        // Check for collisions with colliders (paddle, bricks, walls)
        for (collider_entity, collider_transform, collider, brick, paddle, wall) in collider_query.iter() {
            if check_collision(
                ball_transform.translation,
                ball_collider.size,
                collider_transform.translation,
                collider.size,
            ) {
                // Handle brick collision
                if brick.is_some() {
                    commands.entity(collider_entity).despawn();
                    game_state.score += 10;
                    
                    // Simple reflection - just reverse y velocity for bricks
                    ball.velocity.y = -ball.velocity.y;
                }
                
                // Handle paddle collision with simple bounce
                else if paddle.is_some() {
                    let paddle_pos = collider_transform.translation.truncate();
                    let relative_hit_pos = (ball_position.x - paddle_pos.x) / (PADDLE_SIZE.x / 2.0);
                    
                    // Set new velocity with angle based on hit position
                    let speed = ball.velocity.length();
                    ball.velocity.x = relative_hit_pos * speed;
                    ball.velocity.y = ball.velocity.y.abs(); // Always bounce upward
                }
                
                // Handle wall collision
                else if wall.is_some() {
                    // Simple reflection for walls - flip x velocity for side walls, y for top wall
                    let wall_half_width = collider.size.x / 2.0;
                    let wall_half_height = collider.size.y / 2.0;
                    
                    // If it's a vertical wall (left or right)
                    if wall_half_height > wall_half_width {
                        ball.velocity.x = -ball.velocity.x;
                    } else {
                        ball.velocity.y = -ball.velocity.y;
                    }
                }
            }
        }
    }
}

fn reset_ball(
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
    windows: Query<&Window>,
    mut game_state: ResMut<GameState>,
) {
    let window = windows.single();
    let window_height = window.height();
    
    for (mut ball, mut transform) in ball_query.iter_mut() {
        // Check if ball goes below the screen
        if transform.translation.y < -window_height / 2.0 - BALL_SIZE.y {
            // Reset ball position
            let mut rng = rand::thread_rng();
            let random_angle = rng.gen_range((-std::f32::consts::PI / 4.0)..(std::f32::consts::PI / 4.0));
            let direction = Vec2::new(random_angle.sin(), random_angle.cos()).normalize();

            ball.velocity = direction * BALL_SPEED;
            transform.translation.x = 0.0;
            transform.translation.y = -window_height / 2.0 + 100.0;
            
            // Lose a life
            if game_state.lives > 0 {
                game_state.lives -= 1;
            }
        }
    }
}
