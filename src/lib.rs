use checksum::crc::Crc;
use rustbreak::backend::FileBackend;
use rustbreak::deser::Bincode;
use rustbreak::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Error;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Error")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Error"
    }
}

impl From<std::io::Error> for Error {
    fn from(_e: std::io::Error) -> Self {
        Error
    }
}

impl From<rustbreak::error::RustbreakError> for Error {
    fn from(_e: rustbreak::error::RustbreakError) -> Self {
        Error
    }
}

impl From<&str> for Error {
    fn from(_e: &str) -> Self {
        Error
    }
}

pub type Store = Database<HashMap<u64, Info>, FileBackend, Bincode>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    name: String,
    checksum: u64,
    path: PathBuf,
}

impl Info {
    fn from_entry(entry: fs::DirEntry) -> Result<Info> {
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
    fn save(self, db: &Store) -> Result<()> {
        db.write(|db| db.insert(self.checksum, self))?;
        db.save().map_err(Error::from)
    }
}

trait Information {
    fn checksum(&self) -> Result<u64>;
    fn metadata(&self) -> Result<fs::Metadata>;
}

impl Information for Path {
    fn checksum(&self) -> Result<u64> {
        match self.to_str() {
            Some(filename) => Ok(Crc::new(filename).checksum()?.crc64),
            None => Err(Error),
        }
    }
    fn metadata(&self) -> Result<fs::Metadata> {
        fs::metadata(self).map_err(Error::from)
    }
}

pub fn create_path(p: &Path) -> Result<()> {
    fs::create_dir_all(&p).map_err(Error::from)
}

pub fn create_dir_db(dir: &Path) -> rustbreak::error::Result<Store> {
    let dir = dir.join(".dam.db");
    let db = Store::from_path(dir.as_path(), HashMap::new())?;
    db.load()?;
    Ok(db)
}

pub fn visit_dirs(dir: &Path, db: &Store) -> Result<()> {
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
