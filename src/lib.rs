use checksum::crc::Crc;
use leveldb::database::error::Error;
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options, WriteOptions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    name: String,
    checksum: i32,
    path: PathBuf,
}

impl Info {
    fn from_entry(entry: fs::DirEntry) -> io::Result<Info> {
        let path = entry.path();
        Ok(Info {
            name: match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            checksum: path.checksum()?,
            path: path,
        })
    }
    fn save(&self, db: &Database<i32>) -> Result<(), Error> {
        let write_opts = WriteOptions::new();
        db.put(
            write_opts,
            &self.checksum,
            serde_json::to_vec(&self).unwrap().as_ref(),
        )
    }
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Info> {
        match serde_json::from_slice(bytes) {
            Ok(val) => Ok(val),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
}

trait Information {
    fn checksum(&self) -> io::Result<i32>;
    fn metadata(&self) -> io::Result<fs::Metadata>;
}

impl Information for Path {
    fn checksum(&self) -> io::Result<i32> {
        if let Some(filename) = self.to_str() {
            let mut crc = Crc::new(filename);
            return match crc.checksum() {
                Ok(checksum) => Ok(checksum.crc32 as i32),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            };
        }
        Err(io::Error::new(io::ErrorKind::Other, "No Valid Path"))
    }
    fn metadata(&self) -> io::Result<fs::Metadata> {
        fs::metadata(self)
    }
}

pub fn create_path(p: &Path) -> std::io::Result<()> {
    fs::create_dir_all(&p)
}

pub fn create_dir_db(dir: &Path) -> Database<i32> {
    let dir = dir.join(".db");
    let path = dir.as_path();
    let mut options = Options::new();
    options.create_if_missing = true;
    match Database::<i32>::open(path, options) {
        Ok(db) => db,
        Err(e) => panic!("failed to open database: {:?}", e),
    }
}

pub fn visit_dirs(dir: &Path, db: &Database<i32>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)?.filter(|s| match s {
            Ok(s) => !s.file_name().into_string().unwrap().starts_with("."),
            _ => false,
        }) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let info = Info::from_entry(entry)?;
                match info.save(&db) {
                    Ok(_) => (),
                    Err(e) => panic!("failed to write to database: {:?}", e),
                };
            }
            if path.is_dir() {
                visit_dirs(&path, &db)?;
            }
        }
    }
    Ok(())
}
