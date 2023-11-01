use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Shape2dPlugin::default())
        .insert_resource(GameState::new())
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .add_systems(Update, direction_update)
        .add_systems(Update, snake_update)
        .add_systems(Update, on_edible_eaten)
        .add_systems(PostUpdate, draw_apple)
        .add_systems(PostUpdate, draw_snake)
        .add_event::<TickUpdate>()
        .add_event::<EdibleEaten>()
        .run()
}

#[derive(Event)]
struct TickUpdate;

#[derive(Event)]
struct EdibleEaten;

#[derive(Debug, Clone, Copy)]
enum Tile {
    Void,
    Body,
    Head,
    Tail,
    Apple,
}

#[derive(Debug, Clone, Copy, Component, Eq, PartialEq, PartialOrd, Ord)]
struct Position {
    x: usize,
    y: usize,
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn mov(&self, position: &Position) -> Position {
        match self {
            Self::Left => Position {
                x: position.x - 1,
                y: position.y,
            },
            Self::Right => Position {
                x: position.x + 1,
                y: position.y,
            },
            Self::Up => Position {
                x: position.x,
                y: position.y + 1,
            },
            Self::Down => Position {
                x: position.x,
                y: position.y - 1,
            },
        }
    }
}

#[derive(Debug, Component)]
struct Snake {
    parts: VecDeque<Position>,
    direction: Direction,
    belly: usize,
}

#[derive(Debug, Component)]
struct Edible;

#[derive(Debug, Resource)]
struct GameState {
    width: usize,
    height: usize,
    tick_timer: Timer,
}

impl GameState {
    fn new() -> GameState {
        let width = 10;
        let height = 10;

        GameState {
            width,
            height,
            tick_timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        }
    }
}

fn startup(game_state: Res<GameState>, mut commands: Commands) {
    let GameState { width, height, .. } = *game_state;

    let snake_parts = VecDeque::from(vec![Position::from((width / 2 + 1, height / 2))]);
    let snake = Snake {
        parts: snake_parts,
        direction: Direction::Right,
        belly: 0,
    };

    let apple: Position = (width / 2 + 2, height / 2 + 1).into();

    commands.spawn(Camera2dBundle::default());
    commands.spawn((Edible, apple));
    commands.spawn(snake);
}

fn update(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut tick_event: EventWriter<TickUpdate>,
) {
    game_state.tick_timer.tick(time.delta());

    if game_state.tick_timer.finished() {
        tick_event.send(TickUpdate);
    }
}

fn direction_update(mut snake_query: Query<&mut Snake>, keys: Res<Input<KeyCode>>) {
    let mut snake = snake_query.single_mut();
    if keys.just_pressed(KeyCode::W) {
        snake.direction = Direction::Up;
    }
    if keys.just_pressed(KeyCode::A) {
        snake.direction = Direction::Left;
    }
    if keys.just_pressed(KeyCode::S) {
        snake.direction = Direction::Down;
    }
    if keys.just_pressed(KeyCode::D) {
        snake.direction = Direction::Right;
    }
}

fn on_edible_eaten(mut eaten_events: EventReader<EdibleEaten>, mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(0..10);
    let y = rng.gen_range(0..10);
    for _ in eaten_events.iter() {
        commands.spawn((Edible, Position { x, y }));
    }
}

fn snake_update(
    mut tick_events: EventReader<TickUpdate>,
    mut eaten_events: EventWriter<EdibleEaten>,
    mut snake_query: Query<&mut Snake>,
    apple_query: Query<(Entity, &Edible, &Position)>,
    mut commands: Commands,
) {
    for _ in tick_events.iter() {
        let mut snake = snake_query.single_mut();
        let new_position = snake.direction.mov(snake.parts.front().unwrap());
        snake.parts.push_front(new_position);

        for (apple_entity, _, apple_position) in &apple_query {
            let head_position = snake.parts.front().unwrap();
            if head_position == apple_position {
                snake.belly += 1;

                commands.entity(apple_entity).despawn();
                eaten_events.send(EdibleEaten);
            }
        }

        if snake.belly > 0 {
            snake.belly -= 1
        } else {
            snake.parts.pop_back();
        }
    }
}

fn draw_apple(mut painter: ShapePainter, apple_query: Query<(&Edible, &Position)>) {
    painter.color = Color::RED;
    for (_, position) in &apple_query {
        painter.set_translation(Vec3 {
            x: (position.x * 10) as f32,
            y: (position.y * 10) as f32,
            z: 0.0,
        });
        painter.rect(Vec2 { x: 10.0, y: 10.0 });
    }
}

fn draw_snake(mut painter: ShapePainter, snake_query: Query<&Snake>) {
    painter.color = Color::LIME_GREEN;
    for snake in &snake_query {
        for position in snake.parts.iter() {
            painter.set_translation(Vec3 {
                x: (position.x * 10) as f32,
                y: (position.y * 10) as f32,
                z: 0.0,
            });
            painter.rect(Vec2 { x: 10.0, y: 10.0 });
        }
    }
}
