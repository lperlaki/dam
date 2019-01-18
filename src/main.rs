#![feature(box_syntax)]

mod app;
use std::fs;
use std::io;
use std::path::Path;

fn create_path(p: &Path) -> std::io::Result<()> {
    fs::create_dir_all(&p)
}

fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)?.filter(|s| match s {
            Ok(s) => !s.file_name().into_string().unwrap().starts_with("."),
            _ => false,
        }) {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata()?;
            if path.is_dir() {
                visit_dirs(&path)?;
            }
            println!("{:?}", path);
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    // let matches = app::build().get_matches();
    // create_path(Path::new("./tmp/root/1/2"))
    // fs::rename("./test", "./root/test")
    visit_dirs(Path::new("./tmp"))
}
