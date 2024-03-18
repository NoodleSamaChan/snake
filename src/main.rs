use clap::Parser;
use window_rs::WindowBuffer;
use minifb::{Key, KeyRepeat, Window, WindowOptions, MouseButton, MouseMode};
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};

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

//WORLD CREATION
pub struct World {
    window_buffer: WindowBuffer,
}

impl World {
    pub fn new(
        window_buffer: WindowBuffer,
    ) -> Self {
        Self { window_buffer, 
        }
    }
    
}

//WORLD CREATION END

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut buffer = World::new(
        WindowBuffer::new(cli.width, cli.height)
    );

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
