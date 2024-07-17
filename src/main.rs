use std::{
    fs::File,
    io::Read,
    time::{Duration, Instant},
};

use clap::Parser;
use graphic::{minifb::Minifb, Graphic};
use snake::{
    display, go_display, return_in_time, snake_generator, Cli, Direction, TimeCycle, World,
};
use window_rs::WindowBuffer;

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

    let mut window = Minifb::new("Snake - ESC to exit", buffer.width(), buffer.height());

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
        0,
        None,
        Vec::new(),
        TimeCycle::Forward,
        Some(Vec::new()),
        vec![Direction::Still],
        Some(Vec::new()),
        Direction::Still,
        0,
        0x0033CCFF,
        0x00CC33FF,
        0x0000FF00,
        0x00FF0000,
    );
    game_elements.food_generator(&buffer, &cli);
    snake_generator(&mut game_elements, &buffer, &cli);

    let mut instant = Instant::now();

    while window.is_open() && !window.is_key_down(graphic::Key::Escape) {
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
        window.update_with_buffer(&buffer)
    }

    Ok(())
}
