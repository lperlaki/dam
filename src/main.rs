#![feature(box_syntax, uniform_paths)]
#![allow(unused_variables, dead_code)]

mod app;
mod lib;
use lib::*;
use std::path::Path;

fn main() -> std::io::Result<()> {
    // let matches = app::build().get_matches();
    // create_path(Path::new("./tmp/root/1/2"))
    // fs::rename("./test", "./root/test")
    visit_dirs(Path::new("./tmp"))
}
