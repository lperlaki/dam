#![feature(box_syntax, uniform_paths)]
#![allow(unused_variables, dead_code, unused_imports)]

mod app;
use dam::*;

use std::path::Path;

fn main() -> Result<()> {
    // let matches = app::build().get_matches();
    // create_path(Path::new("./tmp/root/1/2"))
    // fs::rename("./test", "./root/test")
    let path = Path::new("./tmp");
    let database = create_dir_db(path)?;
    visit_dirs(&path, &database)?;
    database.read(|db| {
        println!("{:?}", db.get(&0));
    })?;
    Ok(())
}
