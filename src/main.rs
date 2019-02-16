#![feature(box_syntax, uniform_paths)]
#![allow(unused_variables, dead_code, unused_imports)]

mod app;
use dam::*;
use rusqlite::NO_PARAMS;

use std::path::Path;

fn main() -> Result<()> {
    let matches = app::build().get_matches();

    match matches.subcommand() {
        ("init", Some(matches)) => {
            match Dam::check_path(matches.value_of("DIR")?) {
                DamStatus::Exists(dam) => println!("You are already setup"),
                DamStatus::Empty(path) => {
                    let dam = Dam::init(path);
                    println!("Setup Complete")
                }
            };
        }
        ("list", Some(matches)) => {
            match Dam::check_path(matches.value_of("DIR")?) {
                DamStatus::Exists(dam) => dam.list()?,
                DamStatus::Empty(path) => println!("Please run dam init"),
            };
        }
        ("scan", Some(matches)) => {
            match Dam::check_path(matches.value_of("DIR")?) {
                DamStatus::Exists(dam) => dam.scan()?,
                DamStatus::Empty(path) => println!("Please run dam init"),
            };
        }
        _ => println!("other"),
    }
    Ok(())
}
