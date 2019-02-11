#![feature(box_syntax, uniform_paths)]
#![allow(unused_variables, dead_code, unused_imports)]

mod app;
use dam::*;

use std::path::Path;



fn main() -> Result<()> {
    let matches = app::build().get_matches();
    let path = Path::new(matches.value_of("INPUT")?);
    let database = create_dir_db(path)?;
    visit_dirs(&path, &database)?;
    Ok(())
}
