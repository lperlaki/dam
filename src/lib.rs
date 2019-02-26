#![feature(try_trait, uniform_paths)]

use checksum::crc::Crc;
use chrono::{DateTime, Local};
use image::{DynamicImage, GenericImageView};
use rexif::ExifData;
use rusqlite::types::ToSql;
use rusqlite::{blob::ZeroBlob, Connection, DatabaseName, Row, NO_PARAMS};
use std::fs;
use std::path::{Path, PathBuf};
mod error;
pub use crate::error::Error;

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
        Ok(Self {
            id: path.checksum()?,
            name: entry.file_name().into_string()?,
            path,
            created: DateTime::from(entry.metadata()?.created()?),
        })
    }
    fn from_path(path: &PathBuf) -> Result<Self> {
        Ok(Self {
            id: path.checksum()?,
            name: path.file_name()?.to_os_string().into_string()?,
            path: path.to_path_buf(),
            created: DateTime::from(path.metadata()?.created()?),
        })
    }
    pub fn save(self, conn: &Connection) -> Result<()> {
        // TODO: Check if File is thumbnail Compatible

        let thumb = self.thumbnail()?;

        let pix = thumb.raw_pixels();
        conn.prepare_cached(
            "INSERT INTO entry (id, name, path, created, thumbnail)
                  VALUES (?1, ?2, ?3, ?4, ?5) 
                  ON CONFLICT (id) 
                  DO UPDATE SET name=?2, path=?3, created=?4",
        )?
        .execute(&[
            &self.id,
            &self.name as &ToSql,
            &self.path.to_str() as &ToSql,
            &self.created as &ToSql,
            &ZeroBlob(pix.len() as i32), // TODO: Fix Size (to big)
        ])?;
        let mut blob = conn.blob_open(
            DatabaseName::Main,
            "entry",
            "thumbnail",
            conn.last_insert_rowid(),
            false,
        )?;
        image::jpeg::JPEGEncoder::new(&mut blob)
            .encode(&pix, thumb.width(), thumb.height(), thumb.color())
            .map_err(Error::from)
    }
    pub fn load(row: &Row) -> Self {
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
        conn.prepare("SELECT id, name, path, created FROM entry WHERE name LIKE ?")?
            .query_row(&[format!("%{}%", name)], |row| Entry::load(row))
            .map_err(Error::from)
    }
    pub fn thumbnail(&self) -> Result<DynamicImage> {
        Ok(image::open(&self.path)?.thumbnail(600, 400))
    }
}

fn create_path(p: &Path) -> Result<()> {
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
pub enum DamStatus {
    Empty(PathBuf),
    Exists(Dam),
}

#[derive(Debug)]
pub struct Dam {
    path: PathBuf,
    connection: Connection,
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
            DamStatus::Exists(Dam::init(&path).unwrap())
        } else {
            DamStatus::Empty(path.as_ref().to_path_buf())
        }
    }

    pub fn scan(&self) -> Result<()> {
        self.visit_dirs(&self.path)
    }

    pub fn list(&self) -> Result<()> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, name, path, created FROM entry")?;

        for entry in stmt.query_map(NO_PARAMS, |row| Entry::load(row))? {
            println!("{:?}", entry?);
        }
        Ok(())
    }

    pub fn open(&self, name: &str) -> Result<()> {
        Entry::find(&self.connection, name)?.open()?;
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
                &glob::MatchOptions {
                    case_sensitive: false,
                    require_literal_separator: true,
                    require_literal_leading_dot: true,
                },
            )? {
                let mut info = Entry::from_path(&entry?)?;
                self.sort(&mut info)?;
                info.save(&self.connection)?;
            }
        }
        Ok(())
    }
}
