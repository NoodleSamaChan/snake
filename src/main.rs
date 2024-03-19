use clap::Parser;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use rand::Rng;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use window_rs::WindowBuffer;

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
enum Direction {
    North,
    East,
    West,
    South,
  }

//WORLD CREATION
pub struct World {
    window_buffer: WindowBuffer,
    direction: Direction,
}

impl World {
    pub fn new(window_buffer: WindowBuffer, direction: Direction) -> Self {
        Self {
            window_buffer,
            direction,
        }
    }

    pub fn food_generator(&mut self) -> (usize, usize) {
        loop {
            let x = rand::thread_rng().gen_range(0..self.window_buffer.width());
            let y = rand::thread_rng().gen_range(0..self.window_buffer.height());

            if self.window_buffer[(x, y)] == 0 {
                self.window_buffer[(x, y)] = rgb(0, u8::MAX, 0);
                return (x, y);
            }
        }
    }

    pub fn snake_generator(&mut self) {
        let x_middle_point = self.window_buffer.width() / 2;
        let y_middle_point = self.window_buffer.height() / 2;

        self.window_buffer[(x_middle_point, y_middle_point)] = rgb(0, 0, u8::MAX);
        self.window_buffer[(x_middle_point - 1, y_middle_point)] = rgb(0, 0, u8::MAX);
        self.window_buffer[(x_middle_point - 2, y_middle_point)] = rgb(0, 0, u8::MAX);
    }

    pub fn snake_update(&mut self, food_coordinates: (usize, usize)) {
        self.window_buffer[(food_coordinates.0, food_coordinates.1)] = rgb(0, 0, u8::MAX);
    }

    pub fn handle_user_input(&mut self, window: &Window, cli: &Cli) -> std::io::Result<()> {
        if window.is_key_pressed(Key::Q, KeyRepeat::No) {
            self.window_buffer.reset();
        }

        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            let mut save_file = File::create("save_file")?;

            if cli.file_path != None {
                save_file = File::create(cli.file_path.clone().unwrap())?;
            }
            save_file.write_all(&self.window_buffer.width().to_be_bytes())?;
            save_file.write_all(&self.window_buffer.height().to_be_bytes())?;
            save_file.write_all(&self.speed().to_be_bytes())?;

            for number in &self.window_buffer.buffer() {
                save_file.write_all(&number.to_be_bytes())?;
            }

            save_file.flush()?;
        }

        if window.is_key_pressed(Key::Up, KeyRepeat::No) {
            self.direction = Direction::North
        }

        if window.is_key_pressed(Key::Down, KeyRepeat::No) {
            self.direction = Direction::South
        }

        if window.is_key_pressed(Key::Left, KeyRepeat::No) {
            self.direction = Direction::West
        }

        if window.is_key_pressed(Key::Right, KeyRepeat::No) {
            self.direction = Direction::East
        }

        Ok(())
    }

    pub fn direction(&mut self) {
        let mut next_iteration =
            WindowBuffer::new(self.window_buffer.width(), self.window_buffer.height());

        for x in 0..self.window_buffer.width() {
            for y in 0..self.window_buffer.height() {
                let x = x as isize;
                let y = y as isize;

                match self.direction {
                    Direction::North => {
                        if self.window_buffer.get(x, y + 1) != None {
                            if self.window_buffer[(x as usize, y as usize)] == rgb(0, 0, u8::MAX) {
                                next_iteration[(x as usize, y as usize + 1)] = rgb(0, 0, u8::MAX)
                            }
                        }
                    },
                    Direction::South => {
                        if self.window_buffer.get(x, y - 1) != None {
                            if self.window_buffer[(x as usize, y as usize)] == rgb(0, 0, u8::MAX) {
                                next_iteration[(x as usize, y as usize - 1)] = rgb(0, 0, u8::MAX)
                            }
                        }
                    },
                    Direction::East => {
                        if self.window_buffer.get(x + 1, y) != None {
                            if self.window_buffer[(x as usize, y as usize)] == rgb(0, 0, u8::MAX) {
                                next_iteration[(x as usize + 1, y as usize - 1)] = rgb(0, 0, u8::MAX)
                            }
                        }
                    },
                    Direction::West => {
                        if self.window_buffer.get(x - 1, y) != None {
                            if self.window_buffer[(x as usize, y as usize)] == rgb(0, 0, u8::MAX) {
                                next_iteration[(x as usize - 1, y as usize - 1)] = rgb(0, 0, u8::MAX)
                            }
                        }
                    }
                }
            }
        }
        self.window_buffer = next_iteration;
    }
}
//WORLD CREATION END

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut buffer = World::new(WindowBuffer::new(cli.width, cli.height), Direction::East);

    if cli.file_path != None {
        buffer.window_buffer.reset();

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

        for y in 0..buffer.window_buffer.height() {
            for x in 0..buffer.window_buffer.width() {
                save_file.read_exact(&mut saved_chunk_2)?;
                buffer.window_buffer[(x, y)] = u32::from_be_bytes(saved_chunk_2)
            }
        }
    }

    let mut window = Window::new(
        "Test - ESC to exit",
        buffer.window_buffer.width(),
        buffer.window_buffer.height(),
        WindowOptions {
            scale: minifb::Scale::X8,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        todo!();

        window
            .update_with_buffer(&buffer.window_buffer.buffer(), cli.width, cli.height)
            .unwrap();
    }

    Ok(())
}
