use crate::{create_path, Error, Information, Result};
use chrono::{DateTime, Local};
use image::{DynamicImage, GenericImageView};
use rusqlite::{blob::ZeroBlob, types::ToSql, Connection, DatabaseName, Row};
use std::fs;
use std::path::{Path, PathBuf};

pub type EntryId = u32;

#[derive(Debug)]
pub struct Entry {
    pub id: EntryId,
    pub name: String,
    pub path: PathBuf,
    pub created: DateTime<Local>,
}

impl Entry {
    pub fn from_entry(entry: fs::DirEntry) -> Result<Self> {
        let path = entry.path();
        Ok(Self {
            id: path.checksum()?,
            name: entry.file_name().into_string()?,
            path,
            created: DateTime::from(entry.metadata()?.created()?),
        })
    }
    pub fn from_path(path: &PathBuf) -> Result<Self> {
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
            &self.name as &dyn ToSql,
            &self.path.to_str() as &dyn ToSql,
            &self.created as &dyn ToSql,
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
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            path: row.get::<usize, String>(2).map(PathBuf::from).unwrap(),
            created: row.get::<usize, DateTime<Local>>(3).unwrap(),
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
            .query_row(&[format!("%{}%", name)], |row| Ok(Entry::load(row)))
            .map_err(Error::from)
    }
    pub fn thumbnail(&self) -> Result<DynamicImage> {
        Ok(image::open(&self.path)?.thumbnail(600, 400))
    }
}
