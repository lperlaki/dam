#![feature(box_syntax, uniform_paths)]
#![allow(unused_variables, dead_code, unused_imports)]

mod app;
mod lib;
use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use lib::*;
use std::path::Path;

fn main() -> std::io::Result<()> {
    // let matches = app::build().get_matches();
    // create_path(Path::new("./tmp/root/1/2"))
    // fs::rename("./test", "./root/test")
    let path = Path::new("./tmp");
    let database = create_dir_db(&path);
    visit_dirs(&path, &database)?;
    let read_opts = ReadOptions::new();
    let iter = database.iter(read_opts);
    let entrys = iter
        .map(|val| Info::from_bytes(val.1.as_ref()).unwrap())
        .collect::<Vec<Info>>();
    println!("{:?}", entrys);
    Ok(())
}
