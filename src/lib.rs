#![feature(try_trait)]

use checksum::crc::Crc;
use rexif::ExifData;
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

impl From<rexif::ExifError> for Error {
    fn from(_e: rexif::ExifError) -> Self {
        Error
    }
}

impl From<&str> for Error {
    fn from(_e: &str) -> Self {
        Error
    }
}

impl From<std::option::NoneError> for Error {
    fn from(_e: std::option::NoneError) -> Self {
        Error
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub type Store = Database<HashMap<u64, Entry>, FileBackend, Bincode>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    id: u64,
    name: String,
    pub path: PathBuf,
    exif: HashMap<String, String>,
}

impl Entry {
    fn from_entry(entry: fs::DirEntry) -> Result<Entry> {
        let path = entry.path();
        Ok(Entry {
            id: path.checksum()?,
            name: match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            exif: path
                .clone()
                .exif()?
                .entries
                .iter()
                .map(|entry| (entry.clone().unit, entry.clone().value_more_readable))
                .collect(),
            path: path,
        })
    }
    pub fn save(self, db: &Store) -> Result<()> {
        db.write(|db| db.insert(self.id, self))?;
        db.save().map_err(Error::from)
    }
    // pub fn load(id: u64, db: &Store) -> Result<Option<Entry>> {
    //     db.write(|db| match db.entry(id) {
    //         std::collections::hash_map::Entry::Occupied(v) => Some(*v.get()),
    //         std::collections::hash_map::Entry::Vacant(_) => None,
    //     }).map_err(Error::from)
    // }
    // pub fn load<'a>(id: &'a u64, db: &Store) -> Result<&'a Entry> {
    //     db.read(|db| db.get(id).unwrap()).map_err(Error::from)
    // }
    pub fn rename(&mut self, dest: &Path) -> Result<()> {
        create_path(dest.parent()?)?;
        fs::rename(&self.path, dest)?;
        self.path = dest.to_path_buf();
        Ok(())
    }
}

pub trait Information {
    fn checksum(&self) -> Result<u64>;
    fn metadata(&self) -> Result<fs::Metadata>;
    fn exif(&self) -> Result<ExifData>;
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
    fn exif(&self) -> Result<ExifData> {
        rexif::parse_file(self.to_str()?).map_err(Error::from)
    }
}

pub fn create_path(p: &Path) -> Result<()> {
    fs::create_dir_all(&p).map_err(Error::from)
}

pub fn create_dir_db(dir: &Path) -> Result<Store> {
    let dir = dir.join(".dam.db");
    let db = Store::from_path(dir.as_path(), HashMap::new())?;
    db.load()
        .or_else(|_| db.save())
        .and(Ok(db))
        .map_err(Error::from)
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
                let info = Entry::from_entry(entry)?;
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
