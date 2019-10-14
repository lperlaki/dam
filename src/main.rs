#![feature(box_syntax)]
#![allow(unused_variables, dead_code, unused_imports)]

use dam::*;
use rusqlite::NO_PARAMS;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

fn parse_path(val: &std::ffi::OsStr) -> std::result::Result<PathBuf, std::ffi::OsString> {
    match PathBuf::from(val) {
        path if path.is_dir() => Ok(path),
        _ => Err("Must be a valid directory!".into()),
    }
}

#[derive(StructOpt)]
#[structopt(author, about)]
pub struct Opt {
    #[structopt(short, long,
        default_value_os=std::path::Component::CurDir.as_os_str(),
        help="Sets the DAM home dir",
        parse(try_from_os_str = parse_path))]
    dir: PathBuf,
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt)]
pub enum Cmd {
    #[structopt(about = "init folder as dam")]
    Init,
    #[structopt(about = "list all files")]
    List,
    #[structopt(about = "scan for new")]
    Scan,
    #[structopt(about = "open for new")]
    Open { name: String },
}

fn main() -> Result<()> {
    let matches = Opt::from_args();

    match matches.cmd {
        Cmd::Init => {
            match Dam::check_path(matches.dir) {
                DamStatus::Exists(dam) => println!("You are already setup"),
                DamStatus::Empty(path) => {
                    let dam = Dam::init(path);
                    println!("Setup Complete")
                }
            };
        }
        Cmd::List => {
            match Dam::check_path(matches.dir) {
                DamStatus::Exists(dam) => dam.list()?,
                DamStatus::Empty(path) => println!("Please run dam init"),
            };
        }
        Cmd::Scan => {
            match Dam::check_path(matches.dir) {
                DamStatus::Exists(dam) => dam.scan()?,
                DamStatus::Empty(path) => println!("Please run dam init"),
            };
        }
        Cmd::Open { name } => {
            match Dam::check_path(matches.dir) {
                DamStatus::Exists(dam) => dam.open(&name)?,
                DamStatus::Empty(path) => println!("Please run dam init"),
            };
        }
    }
    Ok(())
}
