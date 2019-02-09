use checksum::crc::Crc;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
struct Info {
    name: std::ffi::OsString,
    checksum: u64,
    filetype: fs::FileType,
    meta: fs::Metadata,
    path: Box<Path>,
}

impl Info {
    fn from_entry(entry: fs::DirEntry) -> io::Result<Info> {
        let path = entry.path();
        Ok(Info {
            name: entry.file_name(),
            checksum: path.checksum()?,
            filetype: entry.file_type()?,
            meta: path.metadata()?,
            path: path.into_boxed_path(),
        })
    }
}

trait Information {
    fn checksum(&self) -> io::Result<u64>;
    fn metadata(&self) -> io::Result<fs::Metadata>;
}

impl Information for Path {
    fn checksum(&self) -> io::Result<u64> {
        if let Some(filename) = self.to_str() {
            let mut crc = Crc::new(filename);
            return match crc.checksum() {
                Ok(checksum) => Ok(checksum.crc64),
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

pub fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)?.filter(|s| match s {
            Ok(s) => !s.file_name().into_string().unwrap().starts_with("."),
            _ => false,
        }) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                println!("{:?}", Info::from_entry(entry)?);
            }
            if path.is_dir() {
                visit_dirs(&path)?;
            }
        }
    }
    Ok(())
}
