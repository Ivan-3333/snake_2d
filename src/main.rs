extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use rand::Rng;
use std::collections::LinkedList;
use std::iter::FromIterator;

const BACKGROUND_COLOR: [f32; 4] = [0.0, 0.5, 0.2, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GRID_ROWS: i32 = 20;
const GRID_COLUMNS: i32 = 20;
const BODY_SIZE: i32 = 25;
const UPDATE_SPEED: u64 = 6;

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window = make_window(opengl);

    let mut game = make_game(opengl);

    game_loop(&mut game, &mut window);
}

fn make_window(opengl: OpenGL) -> GlutinWindow {
    WindowSettings::new(
        "Snake",
        [
            (GRID_COLUMNS * BODY_SIZE) as u32,
            (GRID_ROWS * BODY_SIZE) as u32,
        ],
    )
    .graphics_api(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap()
}

fn make_game(opengl: OpenGL) -> Game {
    Game {
        gl: GlGraphics::new(opengl),
        snake: Snake::init(),
        food: BodyPart {
            x: GRID_COLUMNS / 2,
            y: GRID_ROWS / 2,
        },
    }
}

fn game_loop(game: &mut Game, window: &mut GlutinWindow) {
    let mut events = Events::new(EventSettings::new()).ups(UPDATE_SPEED);
    while let Some(e) = events.next(window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(_u) = e.update_args() {
            game.update();
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button)
            }
        }
    }
}

struct Game {
    gl: GlGraphics,
    snake: Snake,
    food: BodyPart,
}

impl Game {
    fn render(&mut self, arg: &RenderArgs) {
        self.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(BACKGROUND_COLOR, gl)
        });
        self.snake.render(&mut self.gl, arg);
        self.food.render(&mut self.gl, arg);
    }

    fn update(&mut self) {
        if !self.is_end() {
            if self.snake.check_eat(&self.food) {
                self.snake.grow();
                self.place_food();
            }
            self.snake.update_direction();
        }
    }

    fn place_food(&mut self) {
        let mut free_space: Vec<(i32, i32)> = Vec::new();
        for x in 0..GRID_COLUMNS {
            for y in 0..GRID_ROWS {
                if !self.snake.body.iter().any(|&p| p.x == x && p.y == y) {
                    free_space.push((x, y));
                }
            }
        }
        let mut rng = rand::thread_rng();
        let pos = rng.gen_range(1, free_space.len());
        self.food.x = free_space[pos - 1].0;
        self.food.y = free_space[pos - 1].1;
    }

    fn pressed(&mut self, btn: &Button) {
        let last_direction = self.snake.dir.clone();

        self.snake.dir = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down) if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        };

        if btn == &Button::Keyboard(Key::Space) && self.is_end() {
            self.restart();
        }
    }

    fn is_end(&mut self) -> bool {
        self.snake.collision() || self.snake.out_of_bounds()
    }

    fn restart(&mut self) {
        self.snake = Snake::init();
        self.place_food();
    }
}

struct Snake {
    body: LinkedList<BodyPart>,
    dir: Direction,
}

impl Snake {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        self.body.iter().for_each(|part| part.render(gl, args));
    }

    fn update_direction(&mut self) {
        let mut new_head = (*self.body.front().expect("Snake has no body")).clone();
        match self.dir {
            Direction::Left => new_head.x -= 1,
            Direction::Right => new_head.x += 1,
            Direction::Up => new_head.y -= 1,
            Direction::Down => new_head.y += 1,
        }

        self.body.push_front(new_head);
        self.body.pop_back().unwrap();
    }

    fn grow(&mut self) {
        let mut new_tail = (*self.body.back().expect("Snake has no body")).clone();
        new_tail.x += 1;
        self.body.push_back(new_tail);
    }

    fn check_eat(&mut self, food: &BodyPart) -> bool {
        let head = *self.body.front().expect("Snake has no body");
        head.x == food.x && head.y == food.y
    }

    fn collision(&mut self) -> bool {
        let head = *self.body.front().expect("Snake has no body");
        let mut body_without_head = self.body.clone();
        body_without_head.pop_front();
        body_without_head
            .iter()
            .any(|&p| p.x == head.x && p.y == head.y)
    }

    fn out_of_bounds(&mut self) -> bool {
        let head = *self.body.front().expect("Snake has no body");

        head.x < 0 || head.x > GRID_COLUMNS - 1 || head.y < 0 || head.y > GRID_ROWS - 1
    }

    fn init() -> Snake {
        Snake {
            body: LinkedList::from_iter(
                (vec![BodyPart { x: 0, y: 0 }, BodyPart { x: 0, y: 1 }]).into_iter(),
            ),
            dir: Direction::Right,
        }
    }
}

#[derive(Clone, Copy)]
struct BodyPart {
    x: i32,
    y: i32,
}

impl BodyPart {
    fn square(&self) -> graphics::types::Rectangle {
        graphics::rectangle::square(
            (self.x * BODY_SIZE) as f64,
            (self.y * BODY_SIZE) as f64,
            BODY_SIZE as f64,
        )
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let square = self.square();

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            graphics::rectangle(RED, square, transform, gl);
        });
    }
}

#[derive(Clone, PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down,
}
