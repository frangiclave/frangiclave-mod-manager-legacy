#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

extern crate clap;
extern crate fs_extra;
extern crate regex;
extern crate reqwest;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate tempdir;
extern crate zip;

mod game;
mod patch;
mod repo;

use game::{Game, ModDependency};
use repo::Repo;
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
    // Parse the command-line arguments
    let arguments = clap::App::new("Frangiclave")
        .arg(
            clap::Arg::with_name("game_directory")
                .short("g")
                .long("game_directory")
                .value_name("GAME_DIRECTORY")
                .help("Sets the location of the game directory")
                .takes_value(true),
        )
        .get_matches();
    let game_directory = arguments.value_of("game_directory").unwrap_or(".");
    let mut search_path = PathBuf::from(game_directory);
    show_welcome_message();

    // Try to locate the root of the game directory if we're not already there, then load the game
    // directory information.
    loop {
        let game = Game::new(&search_path);
        if game.is_valid() {
            command_loop(&game);
            break;
        } else {
            search_path = match search_path.parent() {
                Some(parent) => parent,
                None => {
                    eprintln!(
                        "ERROR: Cultist Simulator not detected in directory. This program must be \
                         run from somewhere within your game's directory."
                    );
                    println!("Press Enter key to exit...");
                    let mut stdin = std::io::stdin();
                    stdin.read(&mut [0u8]).unwrap();
                    break;
                }
            }.to_owned();
        }
    }
}

fn show_welcome_message() {
    println!("{}", LOGO);
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
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
        command = command.trim().to_string();
        match command.chars().next() {
            Some(c) => match c {
                'p' => patch_game(game),
                'i' => install_mod(game, command.as_ref()),
                'u' => update_mods(),
                'r' => remove_mod(game, command.as_ref()),
                'x' => break,
                _ => eprintln!(
                    "Invalid command name '{}', must be one of the following: p, i, u, r, x",
                    command
                ),
            },
            None => (),
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
    println!("Patch successful.");
    println!("You will need to run this command again whenever Cultist Simulator is updated.");
}

fn install_mod(game: &Game, command: &str) {
    // Get the mod ID as the only argument
    let args: Vec<&str> = command.split(' ').collect();
    if args.len() != 2 {
        eprintln!("Invalid number of arguments specified. Usage: i <mod_id>");
        return;
    }
    let mod_dependency = args[1];

    // Initialize the repo and install the mod for it
    let repo = match Repo::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to prepare repository: {}", e);
            return;
        }
    };
    let dependency = match ModDependency::parse(mod_dependency) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Invalid dependency specifier: {}", e);
            return;
        }
    };
    match repo.install_mod(game, &dependency) {
        Ok(_) => println!("Successfully installed {}", mod_dependency),
        Err(e) => eprintln!("There was an error installing the mod: {}", e),
    };
}

fn update_mods() {
    println!("Updating mods is not implemented yet.");
}

fn remove_mod(game: &Game, command: &str) {
    // Get the mod ID as the only argument
    let args: Vec<&str> = command.split(' ').collect();
    if args.len() != 2 {
        eprintln!("Invalid number of arguments specified. Usage: r <mod_id>");
        return;
    }
    let mod_id = args[1];

    match game.remove_mod(mod_id) {
        Ok(_) => println!("Successfully removed {}", mod_id),
        Err(e) => eprintln!("There was an error removing the mod: {}", e),
    }
}
