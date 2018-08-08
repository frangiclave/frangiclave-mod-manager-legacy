extern crate tempdir;

mod game;
mod patch;

use game::Game;
use std::io::{Read, Write};
use std::path::PathBuf;

const LOGO: &'static str = r#"
   __                       _      _
  / _|_ __ __ _ _ __   __ _(_) ___| | __ ___   _____
 | |_| '__/ _` | '_ \ / _` | |/ __| |/ _` \ \ / / _ \
 |  _| | | (_| | | | | (_| | | (__| | (_| |\ V /  __/
 |_| |_|  \__,_|_| |_|\__, |_|\___|_|\__,_| \_/ \___|
                      |___/
"#;

fn main() {
    show_welcome_message();

    // Load the game directory information, checking if the working directory is a valid game
    // directory first.
    let game = Game::new(&PathBuf::from("."));
    if !game.is_valid() {
        eprintln!(
            "ERROR: Cultist Simulator not detected in working directory. This program must be run \
             from the root of your game's directory."
        );
        println!("Press Enter key to exit...");
        let mut stdin = std::io::stdin();
        stdin.read(&mut [0u8]).unwrap();
    } else {
        command_loop(&game);
    }
}

fn show_welcome_message() {
    println!("{}", LOGO);
}

fn command_loop(game: &Game) {
    // Show a list of possible actions the first time
    println!("Choose an action to perform:");
    println!("[p] Patch Cultist Simulator");
    println!("[i] Install mod");
    println!("[u] Update mods");
    println!("[r] Remove mod");
    println!("[x] Exit");

    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    loop {
        print!("> ");
        stdout.flush().unwrap();
        let mut command = String::new();
        stdin.read_line(&mut command).unwrap();
        match command.trim() {
            "p" => patch_game(game),
            "i" => install_mod(),
            "u" => update_mods(),
            "r" => remove_mod(),
            "x" => break,
            _ => eprintln!(
                "Invalid command name '{}', must be one of the following: p, i, u, r, x",
                command.trim()
            ),
        }
        println!();
    }
}

fn patch_game(game: &Game) {
    let mut stdout = std::io::stdout();
    print!("Applying latest patch to Cultist Simulator...");
    stdout.flush().unwrap();
    match game.patch_assembly() {
        Ok(_) => println!(" [OK]"),
        Err(e) => {
            println!(" [ERROR]");
            eprintln!("There was an error patching the game assembly: {}", e);
            return;
        }
    }
    print!("Creating mods folder...");
    stdout.flush().unwrap();
    match game.make_mods_dir() {
        Ok(_) => println!(" [OK]"),
        Err(e) => {
            println!(" [ERROR]");
            eprintln!("There was an error creating the mods folder: {}", e);
            return;
        }
    }
    println!("Patch successful.")
}

fn install_mod() {
    println!("Installing mods is not implemented yet.");
}

fn update_mods() {
    println!("Updating mods is not implemented yet.");
}

fn remove_mod() {
    println!("Removing mods is not implemented yet.");
}