#![feature(try_trait)]

mod entry;
mod error;

use checksum::crc::Crc;

pub use crate::entry::{Entry, EntryId};
pub use crate::error::Error;
use rexif::ExifData;
use rusqlite::{Connection, NO_PARAMS};
use std::fs;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

pub trait Information {
    fn checksum(&self) -> Result<EntryId>;
    fn exif(&self) -> Result<ExifData>;
}

impl Information for Path {
    fn checksum(&self) -> Result<EntryId> {
        match self.to_str() {
            Some(filename) => Ok(Crc::new(filename).checksum()?.crc32),
            None => Err(Error::new("")),
        }
    }
    fn exif(&self) -> Result<ExifData> {
        rexif::parse_file(self.to_str()?).map_err(Error::from)
    }
}

pub fn create_path(p: &Path) -> Result<()> {
    fs::create_dir_all(&p).map_err(Error::from)
}

fn clear_dir(p: &Path) -> Result<()> {
    match fs::remove_dir(p) {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

fn db_path(path: &PathBuf) -> PathBuf {
    path.join(".dam")
}

fn create_dir_db(path: &PathBuf) -> Result<Connection> {
    Connection::open(db_path(path)).map_err(Error::from)
}

fn init_tables(conn: &Connection) -> Result<usize> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entry (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  path            TEXT NOT NULL,
                  created         DATETIME,
                  thumbnail       BLOB,
                  type            TEXT
                )",
        NO_PARAMS,
    )
    .map_err(Error::from)
}

#[derive(Debug)]
struct Collection {
    contains: Vec<CollectionEntry>,
    path: PathBuf,
    id: EntryId,
}

#[derive(Debug)]
enum CollectionEntry {
    Entry(Entry),
    Collection(Collection),
}

#[derive(Debug)]
pub enum DamStatus {
    Empty(PathBuf),
    Exists(Dam),
}

#[derive(Debug)]
pub struct Dam {
    path: PathBuf,
    db: Connection,
}

impl Dam {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = create_dir_db(&path.as_ref().to_path_buf())?;
        init_tables(&conn)?;
        Ok(Dam {
            path: path.as_ref().to_path_buf(),
            db: conn,
        })
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = create_dir_db(&path.as_ref().to_path_buf())?;
        Ok(Dam {
            path: path.as_ref().to_path_buf(),
            db: conn,
        })
    }

    pub fn check_path<P: AsRef<Path>>(path: P) -> DamStatus {
        if db_path(&path.as_ref().to_path_buf()).exists() {
            DamStatus::Exists(Dam::load(&path).unwrap())
        } else {
            DamStatus::Empty(path.as_ref().to_path_buf())
        }
    }

    pub fn scan(&self) -> Result<()> {
        self.visit_dirs(&self.path)
    }

    pub fn list(&self) -> Result<()> {
        let mut stmt = self
            .db
            .prepare("SELECT id, name, path, created FROM entry")?;

        for entry in stmt.query_map(NO_PARAMS, |row| Ok(Entry::load(row)))? {
            println!("{:?}", entry?);
        }
        Ok(())
    }

    pub fn open(&self, name: &str) -> Result<()> {
        Entry::find(&self.db, name)?.open()?;
        Ok(())
    }

    fn sort(&self, entry: &mut Entry) -> Result<()> {
        let mut path = PathBuf::from(&self.path);
        path.push(&entry.created.format("%G/%b_%d").to_string());
        path.push(&entry.name);
        entry.rename(&path)?;
        clear_dir(entry.path.parent()?)
    }
    fn visit_dirs(&self, dir: &Path) -> Result<()> {
        if dir.is_dir() {
            std::env::set_current_dir(dir)?;
            for entry in glob::glob_with(
                "**/*",
                glob::MatchOptions {
                    case_sensitive: false,
                    require_literal_separator: true,
                    require_literal_leading_dot: true,
                },
            )? {
                let mut info = Entry::from_path(&entry?)?;
                self.sort(&mut info)?;
                info.save(&self.db)?;
            }
        }
        Ok(())
    }
}
