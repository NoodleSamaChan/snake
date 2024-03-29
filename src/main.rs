use crate::Direction::Still;
use clap::{Parser, ValueEnum};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::Rng;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::time::{Duration, Instant};
use window_rs::WindowBuffer;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Difficulty::Medium => write!(f, "medium"),
            Difficulty::Hard => write!(f, "hard"),
            &Difficulty::Easy => write!(f, "easy"),
        }
    }
}

#[derive(PartialEq)]
pub enum TimeCycle {
    Forward,
    Backward,
    Pause,
}

//CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Optional name to operate on
    #[arg(long, default_value_t = 80)]
    width: usize,
    #[arg(long, default_value_t = 50)]
    height: usize,
    #[arg(long, default_value_t = 3)]
    snake_size_start: usize,
    #[arg(long)]
    file_path: Option<String>,
    #[arg(long, default_value_t = 120)]
    snake_speed: usize,
    #[arg(long, default_value_t = Difficulty::Medium)]
    speed_increase: Difficulty,
    #[arg(long, default_value_t = false)]
    bad_berries: bool,
    #[arg(long, default_value_t = false)]
    ghost_mode: bool,
    #[arg(long, default_value_t = false)]
    two_players_mode: bool,
}
//CLI END

//COLOURS MANAGEMENT
pub fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    let a = u32::from(red);
    let b: u32 = u32::from(green);
    let c = u32::from(blue);

    let new_red = a << 16;
    let new_green = b << 8;

    let final_number = new_red | new_green | c;

    return final_number;
}
//COLOURS MANAGEMENT END

pub fn snake_generator(world: &mut World, buffer: &WindowBuffer, cli: &Cli) {
    let x_middle_point = buffer.width() / 2;
    let y_middle_point = buffer.height() / 2;

    world.snake.push((x_middle_point - 2, y_middle_point));
    world.snake.push((x_middle_point - 1, y_middle_point));
    world.snake.push((x_middle_point, y_middle_point));

    if cli.two_players_mode == true {
        println!("LOOP OF SECOND SNAKE");
        world
            .second_snake
            .as_mut()
            .unwrap()
            .push((x_middle_point, y_middle_point - 2));
        world
            .second_snake
            .as_mut()
            .unwrap()
            .push((x_middle_point - 1, y_middle_point - 2));
        world
            .second_snake
            .as_mut()
            .unwrap()
            .push((x_middle_point - 2, y_middle_point - 2));
    }
}

pub fn display(world: &World, buffer: &mut WindowBuffer, cli: &Cli) {
    buffer.reset();
    world
        .snake
        .iter()
        .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = rgb(0, 0, u8::MAX));
    buffer[world.snake[world.snake.len() - 1]] = rgb(u8::MAX, 0, 0);

    if cli.two_players_mode == true && world.second_snake != None {
        world
            .second_snake
            .clone()
            .unwrap()
            .iter()
            .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = rgb(150, 150, 250));
    if let Some(second_snake) = &world.second_snake {
        buffer[second_snake[second_snake.len() - 1]] = rgb(u8::MAX, 0, 0);
    }
    }

    buffer[world.food] = rgb(0, u8::MAX, 0);
    if let Some(pos) = world.bad_berries_position {
        buffer[pos] = rgb(u8::MAX, 0, 0);
    }
}

pub fn go_display(world: &mut World, buffer: &mut WindowBuffer, cli: &Cli) {
    buffer.reset();
    world
        .snake
        .iter()
        .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = rgb(u8::MAX, 0, 0));

    if cli.two_players_mode == true && world.second_snake != None {
        world
            .second_snake
            .clone()
            .unwrap()
            .iter()
            .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = rgb(u8::MAX, 0, 0));
    }

    buffer[world.food] = rgb(u8::MAX, 0, 0);
}

pub fn return_in_time(world: &mut World, cli: &Cli) {
    if world.reversed_snake.is_empty() == false {
        let mut time_turning_snake: Vec<(usize, usize)> = Vec::new();

    let mut previous_position = world.reversed_snake.pop();

    if previous_position.unwrap() == world.snake[0] {
        previous_position = world.reversed_snake.pop();
    }

    time_turning_snake = world
        .snake
        .windows(2)
        .rev()
        .map(|x| x[0])
        .collect::<Vec<_>>();
    time_turning_snake = time_turning_snake.into_iter().rev().collect();

    if let Some(pos) = previous_position {
        time_turning_snake.insert(0, pos);
    }

    world.snake = time_turning_snake;
    }
    

    if cli.two_players_mode == true && world.reversed_second_snake != None {
        let mut time_turning_second_snake: Vec<(usize, usize)> = Vec::new();

        let mut previous_position_second_snake = world.reversed_second_snake.clone().unwrap().pop();

        if previous_position_second_snake.unwrap() == world.second_snake.clone().unwrap()[0] {
            previous_position_second_snake = world.reversed_second_snake.clone().unwrap().pop();
        }

        time_turning_second_snake = world
            .second_snake
            .clone()
            .unwrap()
            .windows(2)
            .rev()
            .map(|x| x[0])
            .collect::<Vec<_>>();
        time_turning_second_snake = time_turning_second_snake.into_iter().rev().collect();

        if let Some(pos) = previous_position_second_snake {
            time_turning_second_snake.insert(0, pos);
        }

        world.second_snake = Some(time_turning_second_snake);
    }
}

#[derive(PartialEq)]
pub enum Direction {
    Still,
    North,
    East,
    West,
    South,
}

//WORLD CREATION
pub struct World {
    direction: Direction,
    first_snake_directions : Vec<Direction>,
    snake: Vec<(usize, usize)>,
    food: (usize, usize),
    finished: bool,
    small_break_timer: Instant,
    space_count: usize,
    snake_speed: usize,
    score: usize,
    bad_berries: (usize, usize),
    bad_berries_position: Option<(usize, usize)>,
    reversed_snake: Vec<(usize, usize)>,
    time_cycle: TimeCycle,
    second_snake: Option<Vec<(usize, usize)>>,
    second_snake_directions : Vec<Direction>,
    reversed_second_snake: Option<Vec<(usize, usize)>>,

}

impl World {
    pub fn new(
        direction: Direction,
        first_snake_directions : Vec<Direction>,
        snake: Vec<(usize, usize)>,
        food: (usize, usize),
        finished: bool,
        small_break_timer: Instant,
        space_count: usize,
        snake_speed: usize,
        score: usize,
        bad_berries: (usize, usize),
        bad_berries_position: Option<(usize, usize)>,
        reversed_snake: Vec<(usize, usize)>,
        time_cycle: TimeCycle,
        second_snake: Option<Vec<(usize, usize)>>,
        second_snake_directions : Vec<Direction>,
        reversed_second_snake: Option<Vec<(usize, usize)>>,
    ) -> Self {
        Self {
            direction,
            first_snake_directions,
            snake,
            food,
            finished,
            small_break_timer,
            space_count,
            snake_speed,
            score,
            bad_berries,
            bad_berries_position,
            reversed_snake,
            time_cycle,
            second_snake,
            second_snake_directions,
            reversed_second_snake,
        }
    }

    pub fn update(&mut self, buffer: &mut WindowBuffer, cli: &Cli) {
        if self.space_count % 2 == 0 {
            self.direction(buffer, cli);
            self.snake_update(buffer, cli);
        }
    }

    pub fn reset(&mut self) {
        self.snake.clear();
    }

    pub fn food_generator(&mut self, buffer: &WindowBuffer, cli: &Cli) {
        loop {
            let x = rand::thread_rng().gen_range(0..buffer.width());
            let y = rand::thread_rng().gen_range(0..buffer.height());
            let v: usize = rand::thread_rng().gen_range(0..buffer.width());
            let w: usize = rand::thread_rng().gen_range(0..buffer.height());

            let checker_1 = self.snake.iter().any(|(a, b)| (a, b) == (&x, &y));
            let checker_2 = self.snake.iter().any(|(a, b)| (a, b) == (&v, &w));

            if checker_1 == true || (x == v && y == w) || checker_2 == true {
                continue;
            } else {
                self.food = (x, y);
                if cli.bad_berries == true {
                    self.bad_berries_position = Some((v, w));
                }
                return;
            }
        }
    }

    pub fn display(&self, buffer: &mut WindowBuffer) {
        buffer.reset();
        self.snake
            .iter()
            .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = rgb(0, 0, u8::MAX));
        buffer[self.snake[self.snake.len() - 1]] = rgb(u8::MAX, 0, 0);

        for x in 0..buffer.width() {
            for y in 0..buffer.height() {
                if (x, y) == (self.food.0, self.food.1)
                    && self.snake[self.snake.len() - 1] != (x, y)
                {
                    buffer[(x, y)] = rgb(0, u8::MAX, 0);
                }
                if (self.bad_berries_position != None)
                    && (x, y)
                        == (
                            self.bad_berries_position.unwrap().0,
                            self.bad_berries_position.unwrap().1,
                        )
                    && self.snake[self.snake.len() - 1] != (x, y)
                {
                    buffer[(x, y)] = rgb(u8::MAX, 0, 0);
                }
            }
        }
    }

    pub fn go_display(&self, buffer: &mut WindowBuffer) {
        buffer.reset();
        self.snake
            .iter()
            .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = rgb(u8::MAX, 0, 0));

        for x in 0..buffer.width() {
            for y in 0..buffer.height() {
                if (x, y) == (self.food.0, self.food.1)
                    && self.snake[self.snake.len() - 1] != (x, y)
                {
                    buffer[(x, y)] = rgb(u8::MAX, 0, 0);
                }
            }
        }
    }

    pub fn handle_user_input(
        &mut self,
        window: &Window,
        cli: &Cli,
        buffer: &WindowBuffer,
    ) -> std::io::Result<()> {
        if window.is_key_pressed(Key::Q, KeyRepeat::No) {
            self.reset();
        }

        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            let mut save_file = File::create("save_file")?;

            if cli.file_path != None {
                save_file = File::create(cli.file_path.clone().unwrap())?;
            }
            save_file.write_all(&buffer.width().to_be_bytes())?;
            save_file.write_all(&buffer.height().to_be_bytes())?;
            save_file.write_all(&self.food.0.to_be_bytes())?;
            save_file.write_all(&self.food.1.to_be_bytes())?;

            /*save_file.write_all(&self.speed().to_be_bytes())?;
            save_file.write_all(&self.snake.to_be_bytes())?;

            for number in &self.snake {
                save_file.write_all(&number.to_be_bytes())?;
            } */

            save_file.flush()?;
        }

        if window.is_key_pressed(Key::Up, KeyRepeat::Yes) {
            if self.first_snake_directions[self.first_snake_directions.len() - 1] != Direction::South {
                self.direction = Direction::North;
            }
        }

        if window.is_key_pressed(Key::Down, KeyRepeat::Yes) {
            if self.first_snake_directions[self.first_snake_directions.len() - 1] != Direction::North {
                self.direction = Direction::South;
            }
        }

        if window.is_key_pressed(Key::Left, KeyRepeat::Yes) {
            if self.first_snake_directions[self.first_snake_directions.len() - 1] != Direction::East {
                self.direction = Direction::West;
            }
        }

        if window.is_key_pressed(Key::Right, KeyRepeat::Yes) {
            if self.first_snake_directions[self.first_snake_directions.len() - 1] != Direction::West {
                self.direction = Direction::East;
            }
        }

        if window.is_key_pressed(Key::U, KeyRepeat::Yes) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1] != Direction::South {
                    self.direction = Direction::North;
                }
            }
        }

        if window.is_key_pressed(Key::N, KeyRepeat::Yes) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1] != Direction::North {
                    self.direction = Direction::South;
                }
            }
        }

        if window.is_key_pressed(Key::H, KeyRepeat::Yes) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1] != Direction::East {
                    self.direction = Direction::West;
                }
            }
        }

        if window.is_key_pressed(Key::J, KeyRepeat::Yes) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1] != Direction::West {
                    self.direction = Direction::East;
                }
            }
        }

        let small_break = Duration::from_millis(0);
        if self.small_break_timer.elapsed() >= small_break {
            window.get_keys_released().iter().for_each(|key| match key {
                Key::Space => self.space_count += 1,
                _ => (),
            });
            self.small_break_timer = Instant::now();
        }

        if window.is_key_pressed(Key::R, KeyRepeat::Yes) {
            self.time_cycle = TimeCycle::Backward;
        }

        if window.is_key_pressed(Key::F, KeyRepeat::Yes) {
            self.time_cycle = TimeCycle::Forward;
        }

        Ok(())
    }

    pub fn snake_update(&mut self, buffer: &WindowBuffer, cli: &Cli) {
        let mut reversed_vector: Vec<(usize, usize)> = Vec::new();

        let head = self.snake[self.snake.len() - 1];
        let mut snake_body = self.snake.clone();
        snake_body.pop();
        let checker = snake_body.iter().any(|(a, b)| (a, b) == (&head.0, &head.1));

        if self.snake[self.snake.len() - 1] == self.food {
            match self.direction {
                Direction::North => {
                    if buffer.get(head.0 as isize, head.1 as isize - 1) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, head.1 - 1));

                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);

                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::North);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::North);
                        }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, buffer.height() - 1));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;

                            self.first_snake_directions.push(Direction::North);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::North);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::South => {
                    if buffer.get(head.0 as isize, head.1 as isize + 1) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, head.1 + 1));

                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);
                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::South);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::South);
                        }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, 0));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;

                            self.first_snake_directions.push(Direction::South);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::South);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::East => {
                    if buffer.get(head.0 as isize + 1, head.1 as isize) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0 + 1, head.1));

                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);
                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::East);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::East);
                            }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((0, head.1));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;
                            self.first_snake_directions.push(Direction::East);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::East);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::West => {
                    if buffer.get(head.0 as isize - 1, head.1 as isize) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0 - 1, head.1));

                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);
                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::West);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::West);
                            }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((buffer.width() - 1, head.1));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;

                            self.first_snake_directions.push(Direction::West);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::West);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::Still => {
                    reversed_vector = self.snake.clone();
                    self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                }
            }
            self.snake = reversed_vector;
        } else if (self.bad_berries_position != None)
            && (self.snake[self.snake.len() - 1] == self.bad_berries_position.unwrap())
        {
            self.bad_berries.0 += 1;

            if self.bad_berries.0 % 2 != 0 {
                self.snake_speed = self.snake_speed / 3;
            } else if (self.bad_berries.0 > 1) && (self.bad_berries.0 % 2 == 0) {
                self.snake_speed = self.snake_speed * 3;
            }

            match self.direction {
                Direction::North => {
                    if buffer.get(head.0 as isize, head.1 as isize - 1) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, head.1 - 1));

                        self.food_generator(&buffer, cli);
                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::North);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::North);
                            }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, buffer.height() - 1));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;

                            self.first_snake_directions.push(Direction::North);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::North);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::South => {
                    if buffer.get(head.0 as isize, head.1 as isize + 1) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, head.1 + 1));

                        self.food_generator(&buffer, cli);
                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::South);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::South);
                        }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, 0));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;

                            self.first_snake_directions.push(Direction::South);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::South);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::East => {
                    if buffer.get(head.0 as isize + 1, head.1 as isize) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0 + 1, head.1));

                        self.food_generator(&buffer, cli);
                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::East);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::East);
                        }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((0, head.1));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;

                            self.first_snake_directions.push(Direction::East);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::East);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::West => {
                    if buffer.get(head.0 as isize - 1, head.1 as isize) != None && checker == false
                    {
                        self.snake.push((head.0, head.1));
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0 - 1, head.1));

                        self.food_generator(&buffer, cli);
                        self.score += 10;
                        self.bad_berries.1 += 1;

                        self.first_snake_directions.push(Direction::West);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::West);
                        }
                    } else {
                        if cli.ghost_mode == true && checker == false {
                            self.snake.push((head.0, head.1));
                            reversed_vector = self
                                .snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((buffer.width() - 1, head.1));

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.score += 10;
                            self.bad_berries.1 += 1;

                            self.first_snake_directions.push(Direction::West);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::West);
                            }
                        } else {
                            self.direction = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            println!("Your score is {}", self.score);

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::Still => {
                    reversed_vector = self.snake.clone();
                    self.first_snake_directions.push(Direction::Still);
                    if cli.two_players_mode == true {
                        self.second_snake_directions.push(Direction::Still);
                    }
                }
            }
            self.reversed_snake.push(reversed_vector[0].clone());
            self.snake = reversed_vector;
        }
    }

    pub fn direction(&mut self, buffer: &WindowBuffer, cli: &Cli) {
        let mut reversed_vector: Vec<(usize, usize)> = Vec::new();
        let head = self.snake[self.snake.len() - 1];
        let mut snake_body = self.snake.clone();
        snake_body.pop();

        let checker = snake_body.iter().any(|(a, b)| (a, b) == (&head.0, &head.1));

        match self.direction {
            Direction::North => {
                if buffer.get(head.0 as isize, head.1 as isize - 1) != None && checker == false {
                    reversed_vector = self
                        .snake
                        .windows(2)
                        .rev()
                        .map(|x| x[1])
                        .collect::<Vec<_>>();
                    reversed_vector = reversed_vector.into_iter().rev().collect();
                    reversed_vector.push((head.0, head.1 - 1));
                    if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                        self.snake_speed -= 1;
                    }

                    self.first_snake_directions.push(Direction::North);
                    if cli.two_players_mode == true {
                        self.second_snake_directions.push(Direction::North);
                    }
                } else {
                    if cli.ghost_mode == true && checker == false {
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, buffer.height() - 1));

                        self.first_snake_directions.push(Direction::North);
                        if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::North);
                        }
                    } else {
                        self.direction = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        println!("Your score is {}", self.score);

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::South => {
                if buffer.get(head.0 as isize, head.1 as isize + 1) != None && checker == false {
                    reversed_vector = self
                        .snake
                        .windows(2)
                        .rev()
                        .map(|x| x[1])
                        .collect::<Vec<_>>();
                    reversed_vector = reversed_vector.into_iter().rev().collect();
                    reversed_vector.push((head.0, head.1 + 1));
                    if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                        self.snake_speed -= 1;
                    }

                    self.first_snake_directions.push(Direction::South);
                    if cli.two_players_mode == true {
                        self.second_snake_directions.push(Direction::South);
                    }
                } else {
                    if cli.ghost_mode == true && checker == false {
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, 0));
                        self.first_snake_directions.push(Direction::South);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::South);
                        }
                    } else {
                        self.direction = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        println!("Your score is {}", self.score);

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::East => {
                if buffer.get(head.0 as isize + 1, head.1 as isize) != None && checker == false {
                    reversed_vector = self
                        .snake
                        .windows(2)
                        .rev()
                        .map(|x| x[1])
                        .collect::<Vec<_>>();
                    reversed_vector = reversed_vector.into_iter().rev().collect();
                    reversed_vector.push((head.0 + 1, head.1));
                    if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                        self.snake_speed -= 1;
                    }

                    self.first_snake_directions.push(Direction::East);
                    if cli.two_players_mode == true {
                        self.second_snake_directions.push(Direction::East);
                    }
                } else {
                    if cli.ghost_mode == true && checker == false {
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((0, head.1));

                        self.first_snake_directions.push(Direction::East);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::East);
                        }
                    } else {
                        self.direction = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        println!("Your score is {}", self.score);

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::West => {
                if buffer.get(head.0 as isize - 1, head.1 as isize) != None && checker == false {
                    reversed_vector = self
                        .snake
                        .windows(2)
                        .rev()
                        .map(|x| x[1])
                        .collect::<Vec<_>>();
                    reversed_vector = reversed_vector.into_iter().rev().collect();
                    reversed_vector.push((head.0 - 1, head.1));
                    if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                        self.snake_speed -= 1;
                    }

                    self.first_snake_directions.push(Direction::West);
                    if cli.two_players_mode == true {
                        self.second_snake_directions.push(Direction::West);
                    }
                } else {
                    if cli.ghost_mode == true && checker == false {
                        reversed_vector = self
                            .snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((buffer.width() - 1, head.1));

                        self.first_snake_directions.push(Direction::West);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::West);
                        }
                    } else {
                        self.direction = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        println!("Your score is {}", self.score);

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::Still => {
                self.direction = Still;
                reversed_vector = self.snake.clone();
                self.first_snake_directions.push(Direction::Still);
                if cli.two_players_mode == true {
                    self.second_snake_directions.push(Direction::Still);
                }
            }
        }
        self.reversed_snake.push(reversed_vector[0].clone());
        self.snake = reversed_vector;
    }

    pub fn return_in_time(&mut self) {
        let mut time_turning_snake: Vec<(usize, usize)> = Vec::new();

        let mut previous_position = self.reversed_snake.pop();
        println!("previous position is {:#?}", previous_position);

        if previous_position.unwrap() == self.snake[0] {
            previous_position = self.reversed_snake.pop();
        }

        time_turning_snake = self
            .snake
            .windows(2)
            .rev()
            .map(|x| x[0])
            .collect::<Vec<_>>();
        time_turning_snake = time_turning_snake.into_iter().rev().collect();

        if previous_position != None {
            time_turning_snake.insert(0, previous_position.unwrap());
            //time_turning_snake.push(previous_position.unwrap());
        }
        println!(" time turning snake is {:#?}", time_turning_snake);
        self.snake = time_turning_snake;
    }
}
//WORLD CREATION END

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut buffer: WindowBuffer = WindowBuffer::new(cli.width, cli.height);

    if cli.file_path != None {
        buffer.reset();

        let mut save_file = File::open(cli.file_path.clone().unwrap())?;

        let mut saved_chunk: [u8; 8] = [0; 8];

        save_file.read_exact(&mut saved_chunk)?;
        let new_width = usize::from_be_bytes(saved_chunk);

        if new_width != cli.width {
            panic!("width different from saved width");
        }

        save_file.read_exact(&mut saved_chunk)?;
        let new_height = usize::from_be_bytes(saved_chunk);

        if new_height != cli.height {
            panic!("height different from saved height");
        }

        save_file.read_exact(&mut saved_chunk)?;
        let mut new_food: (usize, usize) = (usize::from_be_bytes(saved_chunk), 0);

        save_file.read_exact(&mut saved_chunk)?;
        new_food.1 = usize::from_be_bytes(saved_chunk);

        /*let mut saved_chunk_2: [u8; 4] = [0; 4];

        for y in 0..buffer.height() {
            for x in 0..buffer.width() {
                save_file.read_exact(&mut saved_chunk_2)?;
                buffer[(x, y)] = u32::from_be_bytes(saved_chunk_2)
            }
        }*/
    }

    let mut window = Window::new(
        "Test - ESC to exit",
        buffer.width(),
        buffer.height(),
        WindowOptions {
            scale: minifb::Scale::X16,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut game_elements: World = World::new(
        Direction::Still,
        vec![Direction::Still],
        Vec::new(),
        (0, 0),
        false,
        Instant::now(),
        0,
        cli.snake_speed,
        0,
        (0, 0),
        None,
        Vec::new(),
        TimeCycle::Forward,
        Some(Vec::new()),
        vec![Direction::Still],
        Some(Vec::new()),
    );
    game_elements.food_generator(&buffer, &cli);
    snake_generator(&mut game_elements, &buffer, &cli);

    let mut instant = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let _ = game_elements.handle_user_input(&window, &cli, &buffer);
        if game_elements.time_cycle == TimeCycle::Forward {
            if game_elements.finished == false {
                let elapsed_time = Duration::from_millis(game_elements.snake_speed as u64);

                if instant.elapsed() >= elapsed_time {
                    game_elements.update(&mut buffer, &cli);
                    instant = Instant::now();
                }
                display(&game_elements, &mut buffer, &cli);
            } else {
                go_display(&mut game_elements, &mut buffer, &cli);
            }
        } else if game_elements.time_cycle == TimeCycle::Backward {
            let elapsed_time = Duration::from_millis(100);

            if instant.elapsed() >= elapsed_time {
                return_in_time(&mut game_elements, &cli);
                instant = Instant::now();
            }
            display(&game_elements, &mut buffer, &cli);
            game_elements.time_cycle = TimeCycle::Pause;
        }
        window
            .update_with_buffer(&buffer.buffer(), cli.width, cli.height)
            .unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::{assert_debug_snapshot, assert_snapshot};

    #[test]
    fn test_rgb() {
        assert_eq!(rgb(0, 0, 0), 0x00_00_00_00);
        assert_eq!(rgb(255, 255, 255), 0x00_ff_ff_ff);
        assert_eq!(rgb(0x12, 0x34, 0x56), 0x00_12_34_56);
    }

    #[test]
    fn snake_moves_east() {
        let cli = Cli::parse();
        let mut world = World::new(
        Direction::Still,
        vec![Direction::Still],
        Vec::new(),
        (0, 0),
        false,
        Instant::now(),
        0,
        cli.snake_speed,
        0,
        (0, 0),
        None,
        Vec::new(),
        TimeCycle::Forward,
        None,
        vec![Direction::Still],
        None,
    );
        let mut buffer: WindowBuffer = WindowBuffer::new(8, 6);
        let mut game_elements: World = World::new(
            Direction::East,
            Vec::new(),
            Vec::new(),
            (0, 0),
            false,
            Instant::now(),
            0,
            100,
            0,
            (0, 0),
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        game_elements.display(&mut buffer);

        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ..###...
        ........
        ........
        "###
        );

        assert_debug_snapshot!(
            game_elements.snake,
        @r###"
        [
            (
                2,
                3,
            ),
            (
                3,
                3,
            ),
            (
                4,
                3,
            ),
        ]
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ...###..
        ........
        ........
        "###
        );

        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                3,
                3,
            ),
            (
                4,
                3,
            ),
            (
                5,
                3,
            ),
        ]
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ....###.
        ........
        ........
        "###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                4,
                3,
            ),
            (
                5,
                3,
            ),
            (
                6,
                3,
            ),
        ]
        "###
        );

        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        .....###
        ........
        ........
        "###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                5,
                3,
            ),
            (
                6,
                3,
            ),
            (
                7,
                3,
            ),
        ]
        "###
        );
    }

    #[test]
    fn snake_moves_north() {
        let cli = Cli::parse();
        let mut buffer: WindowBuffer = WindowBuffer::new(8, 8);
        let mut game_elements: World = World::new(
            Direction::North,
            Vec::new(),
            Vec::new(),
            (0, 0),
            false,
            Instant::now(),
            0,
            100,
            0,
            (0, 0),
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        game_elements.display(&mut buffer);

        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ........
        ..###...
        ........
        ........
        ........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ....#...
        ...##...
        ........
        ........
        ........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ....#...
        ....#...
        ....#...
        ........
        ........
        ........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ....#...
        ....#...
        ....#...
        ........
        ........
        ........
        ........
        "###
        );
    }

    #[test]
    fn snake_moves_south() {
        let cli = Cli::parse();
        let mut buffer: WindowBuffer = WindowBuffer::new(8, 8);
        let mut game_elements: World = World::new(
            Direction::North,
            Vec::new(),
            Vec::new(),
            (0, 0),
            false,
            Instant::now(),
            0,
            100,
            0,
            (0, 0),
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        game_elements.display(&mut buffer);

        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ........
        ..###...
        ........
        ........
        ........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ........
        ...##...
        ....#...
        ........
        ........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ........
        ....#...
        ....#...
        ....#...
        ........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.......
        ........
        ........
        ........
        ........
        ....#...
        ....#...
        ....#...
        "###
        );
    }

    #[test]
    #[should_panic]
    fn snake_moves_west() {
        let cli = Cli::parse();
        let mut buffer: WindowBuffer = WindowBuffer::new(10, 8);
        let mut game_elements: World = World::new(
            Direction::West,
            Vec::new(),
            Vec::new(),
            (0, 0),
            false,
            Instant::now(),
            0,
            100,
            0,
            (0, 0),
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        game_elements.display(&mut buffer);

        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.........
        ..........
        ..........
        ..........
        ...###....
        ..........
        ..........
        ..........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.........
        ..........
        ..........
        ..........
        ....##....
        ..........
        ..........
        ..........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.........
        ..........
        ..........
        ..........
        ...###....
        ..........
        ..........
        ..........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.........
        ..........
        ..........
        ..........
        ..###.....
        ..........
        ..........
        ..........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.........
        ..........
        ..........
        ..........
        .###......
        ..........
        ..........
        ..........
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);

        assert_snapshot!(
            buffer.to_string(),
            @r###"
        #.........
        ..........
        ..........
        ..........
        ###.......
        ..........
        ..........
        ..........
        "###
        );
    }

    #[test]
    fn snake_eats() {
        let cli = Cli::parse();
        let mut buffer: WindowBuffer = WindowBuffer::new(13, 3);
        let mut game_elements: World = World::new(
            Direction::East,
            Vec::new(),
            Vec::new(),
            (8, 1),
            false,
            Instant::now(),
            0,
            100,
            0,
            (0, 0),
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        game_elements.display(&mut buffer);
        game_elements.snake_update(&mut buffer, &cli);

        assert_snapshot!(
            buffer.to_string(),
            @r###"
        .............
        ....###.#....
        .............
        "###
        );

        assert_debug_snapshot!(
            game_elements.snake,
        @r###"
        [
            (
                4,
                1,
            ),
            (
                5,
                1,
            ),
            (
                6,
                1,
            ),
        ]
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        game_elements.snake_update(&mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        .............
        .....####....
        .............
        "###
        );

        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                5,
                1,
            ),
            (
                6,
                1,
            ),
            (
                7,
                1,
            ),
        ]
        "###
        );
        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        game_elements.snake_update(&mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        .............
        .............
        .............
        "###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                7,
                1,
            ),
            (
                8,
                1,
            ),
            (
                8,
                1,
            ),
            (
                9,
                1,
            ),
        ]
        "###
        );

        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        game_elements.snake_update(&mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        ..#..........
        ........###..
        .............
        "###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                8,
                1,
            ),
            (
                8,
                1,
            ),
            (
                9,
                1,
            ),
            (
                10,
                1,
            ),
        ]
        "###
        );

        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        game_elements.snake_update(&mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        ..#..........
        ........####.
        .............
        "###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                8,
                1,
            ),
            (
                9,
                1,
            ),
            (
                10,
                1,
            ),
            (
                11,
                1,
            ),
        ]
        "###
        );

        game_elements.direction(&mut buffer, &cli);
        game_elements.display(&mut buffer);
        game_elements.snake_update(&mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###"
        ..#..........
        .........####
        .............
        "###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###"
        [
            (
                9,
                1,
            ),
            (
                10,
                1,
            ),
            (
                11,
                1,
            ),
            (
                12,
                1,
            ),
        ]
        "###
        );
    }

    #[test]
    fn snake_turns_time() {
        let cli = Cli::parse();
        let mut buffer: WindowBuffer = WindowBuffer::new(13, 3);
        let mut game_elements: World = World::new(
            Direction::East,
            Vec::new(),
            Vec::new(),
            (8, 1),
            false,
            Instant::now(),
            0,
            100,
            0,
            (0, 0),
            None,
            Vec::new(),
            TimeCycle::Backward,
            None,
            Vec::new(),
            None,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        game_elements.display(&mut buffer);
        game_elements.return_in_time();

        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );

        assert_debug_snapshot!(
            game_elements.snake,
        @r###""###
        );
        game_elements.return_in_time();
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );

        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );
        game_elements.return_in_time();
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );

        game_elements.return_in_time();
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );

        game_elements.return_in_time();
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );

        game_elements.return_in_time();
        game_elements.display(&mut buffer);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );
    }
}
