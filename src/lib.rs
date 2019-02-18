#![feature(try_trait, uniform_paths)]

use checksum::crc::Crc;
use chrono::{DateTime, Local};
use rexif::ExifData;
use rusqlite::types::ToSql;
use rusqlite::{Connection, Row, NO_PARAMS};
use std::fs;
use std::path::{Path, PathBuf};
mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

type EntryId = u32;

#[derive(Debug)]
pub struct Entry {
    id: EntryId,
    name: String,
    path: PathBuf,
    created: DateTime<Local>,
}

impl Entry {
    fn from_entry(entry: fs::DirEntry) -> Result<Self> {
        let path = entry.path();
        Ok(Entry {
            id: path.checksum()?,
            name: entry.file_name().into_string()?,
            path: path,
            created: DateTime::from(entry.metadata()?.created()?),
        })
    }
    pub fn save(self, conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO entry (id, name, path, created)
                  VALUES (?1, ?2, ?3, ?4) 
                  ON CONFLICT (id) 
                  DO UPDATE SET name=?2, path=?3, created=?4",
        )?;
        stmt.execute(&[
            &self.id,
            &self.name as &ToSql,
            &self.path.to_str() as &ToSql,
            &self.created as &ToSql,
        ])?;
        Ok(())
    }
    pub fn load(row: &Row) -> Entry {
        Entry {
            id: row.get(0),
            name: row.get(1),
            path: PathBuf::from(row.get::<usize, String>(2)),
            created: row.get::<usize, DateTime<Local>>(3),
        }
    }
    pub fn rename(&mut self, dest: &Path) -> Result<()> {
        create_path(dest.parent()?)?;
        fs::rename(&self.path, dest)?;
        self.path = dest.to_path_buf();
        Ok(())
    }
    pub fn open(&self) -> Result<std::process::ExitStatus> {
        open::that(&self.path).map_err(Error::from)
    }
    pub fn find(conn: &Connection, name: &str) -> Result<Self> {
        let mut stmt =
            conn.prepare("SELECT id, name, path, created FROM entry WHERE name LIKE ?")?;
        stmt.query_row(&[format!("%{}%", name)], |row| Entry::load(row))
            .map_err(Error::from)
    }
}

pub trait Information {
    fn checksum(&self) -> Result<EntryId>;
    fn metadata(&self) -> Result<fs::Metadata>;
    fn exif(&self) -> Result<ExifData>;
}

impl Information for Path {
    fn checksum(&self) -> Result<EntryId> {
        match self.to_str() {
            Some(filename) => Ok(Crc::new(filename).checksum()?.crc32),
            None => Err(Error::new("")),
        }
    }
    fn metadata(&self) -> Result<fs::Metadata> {
        fs::metadata(self).map_err(Error::from)
    }
    fn exif(&self) -> Result<ExifData> {
        rexif::parse_file(self.to_str()?).map_err(Error::from)
    }
}

fn create_path(p: &Path) -> Result<()> {
    fs::create_dir_all(&p).map_err(Error::from)
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
pub struct Dam {
    path: PathBuf,
    connection: Connection,
}

#[derive(Debug)]
pub enum DamStatus {
    Empty(PathBuf),
    Exists(Dam),
}

fn db_path(path: &PathBuf) -> PathBuf {
    path.join(".dam")
}

impl Dam {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = create_dir_db(&path.as_ref().to_path_buf())?;
        init_tables(&conn)?;
        Ok(Dam {
            path: path.as_ref().to_path_buf(),
            connection: conn,
        })
    }

    pub fn check_path<P: AsRef<Path>>(path: P) -> DamStatus {
        if db_path(&path.as_ref().to_path_buf()).exists() {
            return DamStatus::Exists(Dam::init(&path).unwrap());
        } else {
            return DamStatus::Empty(path.as_ref().to_path_buf());
        }
    }

    pub fn scan(&self) -> Result<()> {
        self.visit_dirs(&self.path)
    }

    pub fn list(&self) -> Result<()> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, name, path, created FROM entry")?;
        let entry_iter = stmt.query_map(NO_PARAMS, |row| Entry::load(row))?;

        for entry in entry_iter {
            println!("{:?}", entry?);
        }
        Ok(())
    }

    fn sort(&self, entry: &mut Entry) -> Result<()> {
        let mut path = PathBuf::from(&self.path);
        path.push(&entry.created.format("%G/%b_%d").to_string());
        path.push(&entry.name);
        entry.rename(&path)
    }
    fn visit_dirs(&self, dir: &Path) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)?.filter(|s| match s {
                Ok(s) => !s.file_name().into_string().unwrap().starts_with("."),
                _ => false,
            }) {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let mut info = Entry::from_entry(entry)?;
                    &self.sort(&mut info)?;
                    info.save(&self.connection)?;
                }
                if path.is_dir() {
                    &self.visit_dirs(&path)?;
                }
            }
        }
        Ok(())
    }
    pub fn open(&self, name: &str) -> Result<()> {
        Entry::find(&self.connection, name)?.open()?;
        Ok(())
    }
}
