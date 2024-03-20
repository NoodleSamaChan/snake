use clap::Parser;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::Rng;
use std::clone;
use std::fs::File;
use std::io::{Read, Write};
use window_rs::WindowBuffer;
use std::time::{Instant, Duration};

//CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Optional name to operate on
    #[arg(long, default_value_t = 160)]
    width: usize,
    #[arg(long, default_value_t = 90)]
    height: usize,
    #[arg(long, default_value_t = 3)]
    snake_size_start: usize,
    #[arg(long)]
    file_path: Option<String>,
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

#[derive(PartialEq)]
pub enum Direction {
    North,
    East,
    West,
    South,
  }

//WORLD CREATION
pub struct World {
    direction: Direction,
    snake: Vec<(usize, usize)>,
    food: (usize, usize),
    finished: bool,
}

impl World {
    pub fn new(direction: Direction, snake: Vec<(usize, usize)>, food: (usize, usize), finished: bool) -> Self {
        Self {
            direction,
            snake,
            food,
            finished,
        }
    }

    pub fn food_generator(&mut self, buffer: &WindowBuffer){
        loop {
            let x = rand::thread_rng().gen_range(0..buffer.width());
            let y = rand::thread_rng().gen_range(0..buffer.height());

            let mut checker = self.snake.iter().any(|(a, b)| (a, b) == (&x, &y));

            if checker == true {
                continue
            } else {
                self.food = (x, y);
                println!("food generated");
                return;
            }
        }
    }

    pub fn snake_generator(&mut self, buffer: &WindowBuffer) {
        let x_middle_point = buffer.width() / 2;
        let y_middle_point = buffer.height() / 2;

        self.snake.push((x_middle_point - 2, y_middle_point));
        self.snake.push((x_middle_point - 1, y_middle_point));
        self.snake.push((x_middle_point, y_middle_point));
        println!("snake generated");
    }

    pub fn display(& self, buffer: &mut WindowBuffer) {

        self.snake.iter().for_each(|(x, y)| buffer[(x.clone(), y.clone())] = rgb(0, 0, u8::MAX));
        
        for x in 0..buffer.width(){
            for y in 0..buffer.height(){
                if (x, y) == (self.food.0, self.food.1) {
                    buffer[(x, y)] = rgb(0, u8::MAX, 0);
                    println!("color changed");
                }
            }
        }
    }

    pub fn handle_user_input(&mut self, window: &Window, cli: &Cli, buffer: &mut WindowBuffer) -> std::io::Result<()> {
        if window.is_key_pressed(Key::Q, KeyRepeat::No) {
            buffer.reset();
        }

        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            let mut save_file = File::create("save_file")?;

            if cli.file_path != None {
                save_file = File::create(cli.file_path.clone().unwrap())?;
            }
            save_file.write_all(&buffer.width().to_be_bytes())?;
            save_file.write_all(&buffer.height().to_be_bytes())?;
            //save_file.write_all(&self.speed().to_be_bytes())?;

            for number in &buffer.buffer() {
                save_file.write_all(&number.to_be_bytes())?;
            }

            save_file.flush()?;
        }

        if window.is_key_pressed(Key::Up, KeyRepeat::No) {
            self.direction = Direction::North;
            self.direction(buffer);
        }

        if window.is_key_pressed(Key::Down, KeyRepeat::No) {
            self.direction = Direction::South;
            self.direction(buffer);
        }

        if window.is_key_pressed(Key::Left, KeyRepeat::No) {
            self.direction = Direction::West;
            self.direction(buffer);
        }

        if window.is_key_pressed(Key::Right, KeyRepeat::No) {
            self.direction = Direction::East;
            self.direction(buffer);
        }

        Ok(())
    }
/* 
    pub fn snake_update(&mut self, food_coordinates: (usize, usize)) {

        for x in 0..self.window_buffer.width() {
            for y in 0..self.window_buffer.height() {
                let x = x;
                let y = y;

                if (self.window_buffer[(x, y)] == rgb(0, 0, u8::MAX)) && (self.window_buffer[(x, y)] == self.window_buffer[(food_coordinates.0, food_coordinates.1)]) {
                        self.window_buffer[(food_coordinates.0, food_coordinates.1)] = rgb(0, 0, u8::MAX);
                        self.food_generator();
                }
            }
        }
    }*/

    pub fn direction(&mut self, buffer: &mut WindowBuffer) {
        let mut reversed_vector: Vec<(usize, usize)> = Vec::new();

        for x in 0..buffer.width() {
            for y in 0..buffer.height() {
                let x = x as isize;
                let y = y as isize;

                match self.direction {
                    Direction::North => {
                        if buffer.get(x, y - 1) != None {
                            if (self.snake[self.snake.len() - 1]) == (x as usize, y as usize) {
                                reversed_vector = self.snake.windows(2).rev().map(|x| x[1]).collect::<Vec<_>>();
                                reversed_vector.push((x as usize, y as usize - 1));
                            }
                        } 
                    },
                    Direction::South => {
                        if buffer.get(x, y + 1) != None {
                            if (self.snake[self.snake.len() - 1]) == (x as usize, y as usize) {
                                reversed_vector = self.snake.windows(2).rev().map(|x| x[1]).collect::<Vec<_>>();
                                reversed_vector.push((x as usize, y as usize + 1));
                            }
                        }
                    },
                    Direction::East => {
                        if buffer.get(x + 1, y) != None {
                            if buffer.get(x + 1, y) != None {
                                if (self.snake[self.snake.len() - 1]) == (x as usize, y as usize) {
                                    reversed_vector = self.snake.windows(2).rev().map(|x| x[1]).collect::<Vec<_>>();
                                    reversed_vector.push((x as usize + 1, y as usize));
                                }
                            }
                        }
                    },
                    Direction::West => {
                        if buffer.get(x - 1, y) != None {
                            if buffer.get(x - 1, y) != None {
                                if (self.snake[self.snake.len() - 1]) == (x as usize, y as usize) {
                                    reversed_vector = self.snake.windows(2).rev().map(|x| x[1]).collect::<Vec<_>>();
                                    reversed_vector.push((x as usize - 1, y as usize));
                                }
                            }
                        }
                    }
                }
            }
        }
    self.snake = reversed_vector;
    buffer.reset()
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

        let mut saved_chunk_2: [u8; 4] = [0; 4];

        for y in 0..buffer.height() {
            for x in 0..buffer.width() {
                save_file.read_exact(&mut saved_chunk_2)?;
                buffer[(x, y)] = u32::from_be_bytes(saved_chunk_2)
            }
        }
    }

    let mut window = Window::new(
        "Test - ESC to exit",
        buffer.width(),
        buffer.height(),
        WindowOptions {
            scale: minifb::Scale::X8,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut game_elements: World = World::new(Direction::North, Vec::new(), (0, 0), false);
    game_elements.snake_generator(&buffer);
    game_elements.food_generator(&buffer);

    let mut instant = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let _ = game_elements.handle_user_input(&window, &cli, &mut buffer);
        let two_seconds = Duration::from_secs(1);

        game_elements.display(&mut buffer);
        println!("lunaaaaa");

        window
            .update_with_buffer(&buffer.buffer(), cli.width, cli.height)
            .unwrap();
    }

    Ok(())
}
