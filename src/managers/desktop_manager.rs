use std::{env, fs};
use std::io::Read;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;

use crate::chip8::constants::{
    BLACK_COLOR, SCALE, SCREEN_WIDTH, TICKS_PER_FRAME, WHITE_COLOR, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::chip8::core::Chip8;

pub struct DesktopGameManager {
    sdl_context: Sdl,
    canvas: WindowCanvas,
}

impl DesktopGameManager {
    pub fn new() -> Self {
        let sdl = Self::create_sql();

        Self {
            canvas: Self::create_canvas(&sdl),
            sdl_context: sdl,
        }
    }

    fn choose_game(&self) -> String {
        let args: Vec<_> = env::args().collect();

        if args.len() == 2 {
            return args[1].to_owned();
        }

        let paths = fs::read_dir("./c8games").unwrap();

        println!("Choose a game from the list: ");

        let mut games: Vec<String> = Vec::new();

        let mut i: u8 = 0;

        for path in paths {
            let p = path.unwrap();

            println!("{}- {:?}", i, &p.file_name());

            let game_path = String::from(p.path().to_str().unwrap());

            games.push(game_path);

            i = i + 1;
        }

        loop {
            let mut choice = String::new();

            std::io::stdin().read_line(&mut choice).unwrap();

            let choice: usize = choice.trim().parse().unwrap();

            if choice > games.len() {
                println!("Invalid choice. Please choose a valid game number.");
                continue;
            }

            return games[choice].to_owned();
        }
    }

    fn read_game_rom(&self, path: &String) -> Vec<u8> {
        println!("Loading {path}...");

        let mut rom_file = fs::File::open(path).expect("Failed to open ROM file");

        let mut rom_data = Vec::new();

        rom_file
            .read_to_end(&mut rom_data)
            .expect("Failed to read ROM file");

        return rom_data;
    }

    pub fn start_game_loop(&mut self, mut chip8: Chip8) {
        // get game from args or prompt the user to choose a game
        let game_path = self.choose_game();

        // read the game data from the file
        let game_data = self.read_game_rom(&game_path);

        // load the game into the chip memory
        chip8.load(&game_data);

        // get events from sdl context
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        //setup game loop
        'gameloop: loop {
            for evt in event_pump.poll_iter() {
                match evt {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'gameloop;
                    }

                    //track when key is pressed
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        //only satisfied if value on right matches on
                        //left
                        if let Some(k) = self.key2btn(key) {
                            chip8.keypress(k, true);
                        }
                    }

                    //track when key released
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Some(k) = self.key2btn(key) {
                            chip8.keypress(k, false);
                        }
                    }
                    _ => (),
                }
            }

            for _ in 0..TICKS_PER_FRAME {
                chip8.tick();
            }

            chip8.tick_timers();
            self.draw_screen(&chip8);
        }
    }

    fn create_sql() -> Sdl {
        // Setup SDL
        let sdl_context = sdl2::init().unwrap();

        return sdl_context;
    }

    fn create_canvas(sdl: &Sdl) -> WindowCanvas {
        let video_subsystem = sdl.video().unwrap();

        //create screen according to size and position in center of monitor
        let window = video_subsystem
            .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .expect("Could not create video window");

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .expect("Could not create canvas");

        canvas.set_draw_color(BLACK_COLOR);

        canvas.clear();
        canvas.present();

        return canvas;
    }

    pub fn draw_screen(&mut self, chip8: &Chip8) {
        // Clear canvas as black
        self.canvas.set_draw_color(BLACK_COLOR);
        self.canvas.clear();

        // self.canvas.fill_rect(None).unwrap();

        let screen_buf = chip8.get_screen();

        // Now set draw color to white, iterate through each point and see if it should be drawn
        self.canvas.set_draw_color(WHITE_COLOR);

        for (i, pixel) in screen_buf.iter().enumerate() {
            if *pixel {
                // Convert our 1D array's index into a 2D (x,y) position
                let x = (i % SCREEN_WIDTH) as u32;
                let y = (i / SCREEN_WIDTH) as u32;

                // Draw a rectangle at (x,y), scaled up by our SCALE value
                let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);

                self.canvas.fill_rect(rect).unwrap();
            }
        }

        self.canvas.present();
    }

    fn key2btn(&self, key: Keycode) -> Option<usize> {
        match key {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => None,
        }
    }
}
