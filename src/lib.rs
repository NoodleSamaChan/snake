use crate::Direction::Still;
use clap::{Parser, ValueEnum};
use graphic::Graphic;
use rand::Rng;
use std::fmt;
use std::fs::File;
use std::io::Write;
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
    pub width: usize,
    #[arg(long, default_value_t = 50)]
    pub height: usize,
    #[arg(long, default_value_t = 3)]
    pub snake_size_start: usize,
    #[arg(long)]
    pub file_path: Option<String>,
    #[arg(long, default_value_t = 120)]
    pub snake_speed: usize,
    #[arg(long, default_value_t = Difficulty::Medium)]
    pub speed_increase: Difficulty,
    #[arg(long, default_value_t = false)]
    pub bad_berries: bool,
    #[arg(long, default_value_t = false)]
    pub ghost_mode: bool,
    #[arg(long, default_value_t = false)]
    pub two_players_mode: bool,
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

pub fn snakes_collision_checker(world: &World, cli: &Cli) -> bool {
    if cli.two_players_mode == true {
        let first_snake_head = world.snake[world.snake.len() - 1];
        let mut first_snake_body = world.snake.clone();
        first_snake_body.pop();

        let mut second_snake_head = (0, 0);
        let mut second_snake_body = Vec::new();

        if let Some(second_snake) = &world.second_snake {
            second_snake_head = second_snake[second_snake.len() - 1];
            second_snake_body = second_snake.clone();
        }
        second_snake_body.pop();

        let checker_first_snake_into_second = second_snake_body
            .iter()
            .any(|(a, b)| (a, b) == (&first_snake_head.0, &first_snake_head.1));
        let checker_second_snake_into_first = first_snake_body
            .iter()
            .any(|(a, b)| (a, b) == (&second_snake_head.0, &second_snake_head.1));

        if checker_first_snake_into_second == true {
            return true;
        }
        if checker_second_snake_into_first == true {
            return true;
        }
        if (checker_first_snake_into_second == false) && (checker_second_snake_into_first == false)
        {
            return false;
        }

        unreachable!("Problem with collision checker");
    } else {
        return false;
    }
}

pub fn display(world: &World, buffer: &mut WindowBuffer, cli: &Cli) {
    buffer.reset();
    world
        .snake
        .iter()
        .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = world.first_snake_colour);
    buffer[world.snake[world.snake.len() - 1]] = world.first_snake_colour;

    if cli.two_players_mode == true && world.second_snake != None {
        if let Some(second_snake) = &world.second_snake {
            world
                .second_snake
                .clone()
                .unwrap()
                .iter()
                .for_each(|(x, y)| buffer[(x.clone(), y.clone())] = world.second_snake_colour);
            buffer[second_snake[second_snake.len() - 1]] = world.second_snake_colour;
        }
    }

    buffer[world.food] = world.food_colour;

    if let Some(pos) = world.bad_berries_position {
        buffer[pos] = world.bad_berries_colour;
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
    if world.finished == true {
        world.finished = false
    }
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

#[derive(PartialEq, Debug, Clone)]
pub enum Direction {
    Still,
    North,
    East,
    West,
    South,
}

//WORLD CREATION
pub struct World {
    pub current_direction_first_snake: Direction,
    pub first_snake_directions: Vec<Direction>,
    pub snake: Vec<(usize, usize)>,
    pub food: (usize, usize),
    pub finished: bool,
    pub small_break_timer: Instant,
    pub space_count: usize,
    pub snake_speed: usize,
    pub score: usize,
    pub bad_berries: usize,
    pub bad_berries_position: Option<(usize, usize)>,
    pub reversed_snake: Vec<(usize, usize)>,
    pub time_cycle: TimeCycle,
    pub second_snake: Option<Vec<(usize, usize)>>,
    pub second_snake_directions: Vec<Direction>,
    pub reversed_second_snake: Option<Vec<(usize, usize)>>,
    pub current_direction_second_snake: Direction,
    pub second_score: usize,
    pub first_snake_colour: u32,
    pub second_snake_colour: u32,
    pub food_colour: u32,
    pub bad_berries_colour: u32,
}

impl World {
    pub fn new(
        current_direction_first_snake: Direction,
        first_snake_directions: Vec<Direction>,
        snake: Vec<(usize, usize)>,
        food: (usize, usize),
        finished: bool,
        small_break_timer: Instant,
        space_count: usize,
        snake_speed: usize,
        score: usize,
        bad_berries: usize,
        bad_berries_position: Option<(usize, usize)>,
        reversed_snake: Vec<(usize, usize)>,
        time_cycle: TimeCycle,
        second_snake: Option<Vec<(usize, usize)>>,
        second_snake_directions: Vec<Direction>,
        reversed_second_snake: Option<Vec<(usize, usize)>>,
        current_direction_second_snake: Direction,
        second_score: usize,
        first_snake_colour: u32,
        second_snake_colour: u32,
        food_colour: u32,
        bad_berries_colour: u32,
    ) -> Self {
        Self {
            current_direction_first_snake,
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
            current_direction_second_snake,
            second_score,
            first_snake_colour,
            second_snake_colour,
            food_colour,
            bad_berries_colour,
        }
    }

    pub fn update(&mut self, buffer: &mut WindowBuffer, cli: &Cli) {
        if self.space_count % 2 == 0 {
            self.direction(buffer, cli);
            self.snake_update(buffer, cli);

            if cli.two_players_mode == true {
                self.direction_second_snake(buffer, cli);
                self.second_snake_update(buffer, cli);
            }
        }
    }

    pub fn reset(&mut self) {
        self.snake.clear();
        if let Some(mut pos) = self.second_snake.clone() {
            pos.clear();
        }
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

    pub fn handle_user_input<W: Graphic>(
        &mut self,
        window: &W,
        cli: &Cli,
        buffer: &WindowBuffer,
    ) -> std::io::Result<()> {
        if window.is_key_pressed(graphic::Key::Quit) {
            self.reset();
        }

        if window.is_key_pressed(graphic::Key::Save) {
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

        if window.is_key_pressed(graphic::Key::Up) {
            self.time_cycle = TimeCycle::Forward;
            if self.first_snake_directions[self.first_snake_directions.len() - 1]
                != Direction::South
            {
                self.current_direction_first_snake = Direction::North;
            }
        }

        if window.is_key_pressed(graphic::Key::Down) {
            self.time_cycle = TimeCycle::Forward;
            if self.first_snake_directions[self.first_snake_directions.len() - 1]
                != Direction::North
            {
                self.current_direction_first_snake = Direction::South;
            }
        }

        if window.is_key_pressed(graphic::Key::Left) {
            self.time_cycle = TimeCycle::Forward;
            if self.first_snake_directions[self.first_snake_directions.len() - 1] != Direction::East
            {
                let mut vec_to_check: Vec<Direction> = self.first_snake_directions.clone();
                vec_to_check.dedup();
                if vec_to_check.len() != 1 {
                    self.current_direction_first_snake = Direction::West;
                }
            }
        }

        if window.is_key_pressed(graphic::Key::Right) {
            self.time_cycle = TimeCycle::Forward;
            if self.first_snake_directions[self.first_snake_directions.len() - 1] != Direction::West
            {
                self.current_direction_first_snake = Direction::East;
            }
        }

        if window.is_key_pressed(graphic::Key::UpPlayer2) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1]
                    != Direction::South
                {
                    self.current_direction_second_snake = Direction::North;
                }
            }
        }

        if window.is_key_pressed(graphic::Key::DownPlayer2) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1]
                    != Direction::North
                {
                    self.current_direction_second_snake = Direction::South;
                }
            }
        }

        if window.is_key_pressed(graphic::Key::LeftPlayer2) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1]
                    != Direction::East
                {
                    self.current_direction_second_snake = Direction::West;
                }
            }
        }

        if window.is_key_pressed(graphic::Key::RightPlayer2) {
            if cli.two_players_mode == true {
                if self.second_snake_directions[self.second_snake_directions.len() - 1]
                    != Direction::West
                {
                    let mut vec_to_check_second: Vec<Direction> =
                        self.second_snake_directions.clone();
                    vec_to_check_second.dedup();
                    if vec_to_check_second.len() != 1 {
                        self.current_direction_second_snake = Direction::East;
                    }
                }
            }
        }

        let small_break = Duration::from_millis(0);
        if self.small_break_timer.elapsed() >= small_break {
            window.get_keys_released().iter().for_each(|key| match key {
                graphic::Key::Space => self.space_count += 1,
                _ => (),
            });
            self.small_break_timer = Instant::now();
        }

        if window.is_key_pressed(graphic::Key::Backward) {
            self.time_cycle = TimeCycle::Backward;
        }

        if window.is_key_pressed(graphic::Key::Forward) {
            self.time_cycle = TimeCycle::Forward;
        }

        Ok(())
    }

    pub fn snake_update(&mut self, buffer: &WindowBuffer, cli: &Cli) {
        let snake_collision_check = snakes_collision_checker(&self, cli);
        let mut reversed_vector: Vec<(usize, usize)> = Vec::new();

        let head = self.snake[self.snake.len() - 1];
        let mut snake_body = self.snake.clone();
        snake_body.pop();
        let checker = snake_body.iter().any(|(a, b)| (a, b) == (&head.0, &head.1));

        if self.snake[self.snake.len() - 1] == self.food {
            match self.current_direction_first_snake {
                Direction::North => {
                    if buffer.get(head.0 as isize, head.1 as isize - 1) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::North);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::North);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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

                            self.first_snake_directions.push(Direction::North);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::North);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::South => {
                    if buffer.get(head.0 as isize, head.1 as isize + 1) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::South);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::South);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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

                            self.first_snake_directions.push(Direction::South);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::South);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::East => {
                    if buffer.get(head.0 as isize + 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::East);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::East);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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
                            self.first_snake_directions.push(Direction::East);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::East);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::West => {
                    if buffer.get(head.0 as isize - 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::West);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::West);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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

                            self.first_snake_directions.push(Direction::West);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::West);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

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
            self.bad_berries += 1;

            if self.bad_berries % 2 != 0 {
                self.snake_speed = self.snake_speed / 3;
            } else if (self.bad_berries > 1) && (self.bad_berries % 2 == 0) {
                self.snake_speed = self.snake_speed * 3;
            }

            match self.current_direction_first_snake {
                Direction::North => {
                    if buffer.get(head.0 as isize, head.1 as isize - 1) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::North);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::North);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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

                            self.first_snake_directions.push(Direction::North);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::North);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::South => {
                    if buffer.get(head.0 as isize, head.1 as isize + 1) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::South);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::South);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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

                            self.first_snake_directions.push(Direction::South);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::South);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::East => {
                    if buffer.get(head.0 as isize + 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::East);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::East);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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

                            self.first_snake_directions.push(Direction::East);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::East);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

                            self.first_snake_directions.push(Direction::Still);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::Still);
                            }
                        }
                    }
                }
                Direction::West => {
                    if buffer.get(head.0 as isize - 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
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

                        self.first_snake_directions.push(Direction::West);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::West);
                        }
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
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

                            self.first_snake_directions.push(Direction::West);
                            if cli.two_players_mode == true {
                                self.second_snake_directions.push(Direction::West);
                            }
                        } else {
                            self.current_direction_first_snake = Still;
                            reversed_vector = self.snake.clone();
                            self.finished = true;
                            if cli.two_players_mode == true {
                                println!(
                                    "Player 1 score is {}, Player 2 score is {}",
                                    self.score, self.second_score
                                );
                            } else {
                                println!("Your score is {}", self.score);
                            }

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

    pub fn second_snake_update(&mut self, buffer: &WindowBuffer, cli: &Cli) {
        let snake_collision_check = snakes_collision_checker(&self, cli);

        let mut reversed_vector: Vec<(usize, usize)> = Vec::new();
        let mut head = (0, 0);
        let mut snake_body: Vec<(usize, usize)> = Vec::new();
        if let Some(second_snake) = &self.second_snake {
            head = second_snake[second_snake.len() - 1];
            snake_body = second_snake.clone();
        }
        snake_body.pop();

        let checker = snake_body.iter().any(|(a, b)| (a, b) == (&head.0, &head.1));
        let mut food_check = (0, 0);
        if let Some(second_snake) = self.second_snake.clone() {
            food_check = second_snake[second_snake.len() - 1]
        }

        if food_check == self.food {
            match self.current_direction_second_snake {
                Direction::North => {
                    if buffer.get(head.0 as isize, head.1 as isize - 1) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, head.1 - 1));
                        }
                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);

                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::North);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((head.0, buffer.height() - 1));
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;

                            self.second_snake_directions.push(Direction::North);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::South => {
                    if buffer.get(head.0 as isize, head.1 as isize + 1) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, head.1 + 1));
                        }

                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);
                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::South);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((head.0, 0));
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;

                            self.second_snake_directions.push(Direction::South);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::East => {
                    if buffer.get(head.0 as isize + 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0 + 1, head.1));
                        }

                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);
                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::East);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((0, head.1));
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;
                            self.second_snake_directions.push(Direction::East);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::West => {
                    if buffer.get(head.0 as isize - 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0 - 1, head.1));
                        }

                        if cli.speed_increase == Difficulty::Hard {
                            self.snake_speed = 120;
                        }

                        self.food_generator(&buffer, cli);
                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::West);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((buffer.width() - 1, head.1))
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;

                            self.second_snake_directions.push(Direction::West);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::Still => {
                    if let Some(second_snake) = self.second_snake.clone() {
                        reversed_vector = second_snake.clone();
                    }
                    self.second_snake_directions.push(Direction::Still);
                }
            }
            self.second_snake = Some(reversed_vector);
        } else if (self.bad_berries_position != None)
            && food_check == self.bad_berries_position.unwrap()
        {
            self.bad_berries += 1;

            if self.bad_berries % 2 != 0 {
                self.snake_speed = self.snake_speed / 3;
            } else if (self.bad_berries > 1) && (self.bad_berries % 2 == 0) {
                self.snake_speed = self.snake_speed * 3;
            }

            match self.current_direction_second_snake {
                Direction::North => {
                    if buffer.get(head.0 as isize, head.1 as isize - 1) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, head.1 - 1));
                        }

                        self.food_generator(&buffer, cli);
                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::North);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((head.0, buffer.height() - 1));
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;

                            self.second_snake_directions.push(Direction::North);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::South => {
                    if buffer.get(head.0 as isize, head.1 as isize + 1) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, head.1 + 1));
                        }

                        self.food_generator(&buffer, cli);
                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::South);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((head.0, 0));
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;

                            self.second_snake_directions.push(Direction::South);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::East => {
                    if buffer.get(head.0 as isize + 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0 + 1, head.1));
                        }

                        self.food_generator(&buffer, cli);
                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::East);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((0, head.1));
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;

                            self.second_snake_directions.push(Direction::East);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::West => {
                    if buffer.get(head.0 as isize - 1, head.1 as isize) != None
                        && checker == false
                        && snake_collision_check == false
                    {
                        if let Some(mut second_snake) = self.second_snake.clone() {
                            second_snake.push((head.0, head.1));
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0 - 1, head.1));
                        }

                        self.food_generator(&buffer, cli);
                        self.second_score += 10;

                        self.second_snake_directions.push(Direction::West);
                    } else {
                        if cli.ghost_mode == true
                            && checker == false
                            && snake_collision_check == false
                        {
                            if let Some(mut second_snake) = self.second_snake.clone() {
                                second_snake.push((head.0, head.1));
                                reversed_vector = second_snake
                                    .windows(2)
                                    .rev()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>();
                                reversed_vector = reversed_vector.into_iter().rev().collect();
                                reversed_vector.push((buffer.width() - 1, head.1));
                            }

                            if cli.speed_increase == Difficulty::Hard {
                                self.snake_speed = 120;
                            }

                            self.food_generator(&buffer, cli);

                            self.second_score += 10;

                            self.second_snake_directions.push(Direction::West);
                        } else {
                            self.current_direction_second_snake = Still;
                            if let Some(second_snake) = self.second_snake.clone() {
                                reversed_vector = second_snake.clone();
                            }
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );

                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
                Direction::Still => {
                    if let Some(second_snake) = self.second_snake.clone() {
                        reversed_vector = second_snake.clone();
                    }
                    self.second_snake_directions.push(Direction::Still);
                }
            }

            if let Some(mut reversed_second_snake) = self.reversed_second_snake.clone() {
                reversed_second_snake.push(reversed_vector[0].clone());
            }
            if let Some(mut _second_snake_body) = self.second_snake.clone() {
                self.second_snake = Some(reversed_vector);
            }
        }
    }

    pub fn direction(&mut self, buffer: &WindowBuffer, cli: &Cli) {
        let snake_collision_check = snakes_collision_checker(&self, cli);
        let mut reversed_vector: Vec<(usize, usize)> = Vec::new();
        let head = self.snake[self.snake.len() - 1];
        let mut snake_body = self.snake.clone();
        snake_body.pop();

        let checker = snake_body.iter().any(|(a, b)| (a, b) == (&head.0, &head.1));

        match self.current_direction_first_snake {
            Direction::North => {
                if buffer.get(head.0 as isize, head.1 as isize - 1) != None
                    && checker == false
                    && snake_collision_check == false
                {
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
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
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
                        self.current_direction_first_snake = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        if cli.two_players_mode == true {
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        } else {
                            println!("Your score is {}", self.score);
                        }

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::South => {
                if buffer.get(head.0 as isize, head.1 as isize + 1) != None
                    && checker == false
                    && snake_collision_check == false
                {
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
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
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
                        self.current_direction_first_snake = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        if cli.two_players_mode == true {
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        } else {
                            println!("Your score is {}", self.score);
                        }

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::East => {
                if buffer.get(head.0 as isize + 1, head.1 as isize) != None
                    && checker == false
                    && snake_collision_check == false
                {
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
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
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
                        self.current_direction_first_snake = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        if cli.two_players_mode == true {
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        } else {
                            println!("Your score is {}", self.score);
                        }

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::West => {
                if buffer.get(head.0 as isize - 1, head.1 as isize) != None
                    && checker == false
                    && snake_collision_check == false
                {
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
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
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
                        self.current_direction_first_snake = Still;
                        reversed_vector = self.snake.clone();
                        self.finished = true;
                        if cli.two_players_mode == true {
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        } else {
                            println!("Your score is {}", self.score);
                        }

                        self.first_snake_directions.push(Direction::Still);
                        if cli.two_players_mode == true {
                            self.second_snake_directions.push(Direction::Still);
                        }
                    }
                }
            }
            Direction::Still => {
                self.current_direction_first_snake = Still;
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

    pub fn direction_second_snake(&mut self, buffer: &WindowBuffer, cli: &Cli) {
        let snake_collision_check = snakes_collision_checker(&self, cli);
        let mut reversed_vector: Vec<(usize, usize)> = Vec::new();
        let mut head = (0, 0);
        let mut snake_body: Vec<(usize, usize)> = Vec::new();
        if let Some(second_snake) = &self.second_snake {
            head = second_snake[second_snake.len() - 1];
            snake_body = second_snake.clone();
        }
        snake_body.pop();

        let checker = snake_body.iter().any(|(a, b)| (a, b) == (&head.0, &head.1));

        match self.current_direction_second_snake {
            Direction::North => {
                if buffer.get(head.0 as isize, head.1 as isize - 1) != None
                    && checker == false
                    && snake_collision_check == false
                {
                    if let Some(second_snake) = self.second_snake.clone() {
                        reversed_vector = second_snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, head.1 - 1));
                    }

                    self.second_snake_directions.push(Direction::North);
                } else {
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, buffer.height() - 1));

                            if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                                self.snake_speed -= 1;
                            }
                        }

                        self.second_snake_directions.push(Direction::North);
                    } else {
                        self.current_direction_second_snake = Still;

                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake.clone();
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        }

                        self.second_snake_directions.push(Direction::Still);
                    }
                }
            }
            Direction::South => {
                if buffer.get(head.0 as isize, head.1 as isize + 1) != None
                    && checker == false
                    && snake_collision_check == false
                {
                    if let Some(second_snake) = self.second_snake.clone() {
                        reversed_vector = second_snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0, head.1 + 1));
                        if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                            self.snake_speed -= 1;
                        }
                    }

                    self.second_snake_directions.push(Direction::South);
                } else {
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((head.0, 0));
                            if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                                self.snake_speed -= 1;
                            }
                        }
                        self.second_snake_directions.push(Direction::South);
                    } else {
                        self.current_direction_second_snake = Still;

                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake.clone();
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        }

                        self.second_snake_directions.push(Direction::Still);
                    }
                }
            }
            Direction::East => {
                if buffer.get(head.0 as isize + 1, head.1 as isize) != None
                    && checker == false
                    && snake_collision_check == false
                {
                    if let Some(second_snake) = self.second_snake.clone() {
                        reversed_vector = second_snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0 + 1, head.1));
                        if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                            self.snake_speed -= 1;
                        }
                    }

                    self.second_snake_directions.push(Direction::East);
                } else {
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((0, head.1));
                        }

                        self.second_snake_directions.push(Direction::East);
                    } else {
                        self.current_direction_second_snake = Still;

                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake.clone();
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        }

                        self.second_snake_directions.push(Direction::Still);
                    }
                }
            }
            Direction::West => {
                if buffer.get(head.0 as isize - 1, head.1 as isize) != None
                    && checker == false
                    && snake_collision_check == false
                {
                    if let Some(second_snake) = self.second_snake.clone() {
                        reversed_vector = second_snake
                            .windows(2)
                            .rev()
                            .map(|x| x[1])
                            .collect::<Vec<_>>();
                        reversed_vector = reversed_vector.into_iter().rev().collect();
                        reversed_vector.push((head.0 - 1, head.1));
                        if cli.speed_increase == Difficulty::Hard && self.snake_speed > 0 {
                            self.snake_speed -= 1;
                        }
                    }

                    self.second_snake_directions.push(Direction::West);
                } else {
                    if cli.ghost_mode == true && checker == false && snake_collision_check == false
                    {
                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake
                                .windows(2)
                                .rev()
                                .map(|x| x[1])
                                .collect::<Vec<_>>();
                            reversed_vector = reversed_vector.into_iter().rev().collect();
                            reversed_vector.push((buffer.width() - 1, head.1));
                        }

                        self.second_snake_directions.push(Direction::West);
                    } else {
                        self.current_direction_second_snake = Still;

                        if let Some(second_snake) = self.second_snake.clone() {
                            reversed_vector = second_snake.clone();
                            self.finished = true;
                            println!(
                                "Player 1 score is {}, Player 2 score is {}",
                                self.score, self.second_score
                            );
                        }

                        self.second_snake_directions.push(Direction::Still);
                    }
                }
            }
            Direction::Still => {
                if let Some(second_snake) = self.second_snake.clone() {
                    reversed_vector = second_snake.clone();
                }
                self.second_snake_directions.push(Direction::Still);
            }
        }
        if let Some(mut reversed_second_snake) = self.reversed_second_snake.clone() {
            reversed_second_snake.push(reversed_vector[0].clone());
        }
        if let Some(_second_snake_body) = self.second_snake.clone() {
            self.second_snake = Some(reversed_vector);
        }
    }
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
            0,
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
            Direction::Still,
            0,
            0x0033CCFF,
            0x00CC33FF,
            0x0000FF00,
            0x00FF0000,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        display(&game_elements, &mut buffer, &cli);

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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
            0,
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
            Direction::Still,
            0,
            0x0033CCFF,
            0x00CC33FF,
            0x0000FF00,
            0x00FF0000,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        display(&game_elements, &mut buffer, &cli);

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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
            0,
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
            Direction::Still,
            0,
            0x0033CCFF,
            0x00CC33FF,
            0x0000FF00,
            0x00FF0000,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        display(&game_elements, &mut buffer, &cli);

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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
            0,
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
            Direction::Still,
            0,
            0x0033CCFF,
            0x00CC33FF,
            0x0000FF00,
            0x00FF0000,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        display(&game_elements, &mut buffer, &cli);

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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);

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
            0,
            None,
            Vec::new(),
            TimeCycle::Forward,
            None,
            Vec::new(),
            None,
            Direction::Still,
            0,
            0x0033CCFF,
            0x00CC33FF,
            0x0000FF00,
            0x00FF0000,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
        display(&game_elements, &mut buffer, &cli);
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
            0,
            None,
            Vec::new(),
            TimeCycle::Backward,
            None,
            Vec::new(),
            None,
            Direction::Still,
            0,
            0x0033CCFF,
            0x00CC33FF,
            0x0000FF00,
            0x00FF0000,
        );
        snake_generator(&mut game_elements, &buffer, &cli);
        display(&game_elements, &mut buffer, &cli);
        return_in_time(&mut game_elements, &cli);

        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );

        assert_debug_snapshot!(
            game_elements.snake,
        @r###""###
        );
        return_in_time(&mut game_elements, &cli);
        display(&game_elements, &mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );

        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );
        return_in_time(&mut game_elements, &cli);
        display(&game_elements, &mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );

        return_in_time(&mut game_elements, &cli);
        display(&game_elements, &mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );

        return_in_time(&mut game_elements, &cli);
        display(&game_elements, &mut buffer, &cli);
        assert_snapshot!(
            buffer.to_string(),
            @r###""###
        );
        assert_debug_snapshot!(
            game_elements.snake,
            @r###""###
        );

        return_in_time(&mut game_elements, &cli);
        display(&game_elements, &mut buffer, &cli);
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
