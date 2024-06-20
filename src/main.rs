use managers::desktop_manager::DesktopGameManager;

use crate::chip8::core::Chip8;

pub mod chip8;
pub mod managers;

fn main() {
    // TODO: make webAssembly manager

    let mut game_manager = DesktopGameManager::new();

    let chip8 = Chip8::new();

    game_manager.start_game_loop(chip8);
}
