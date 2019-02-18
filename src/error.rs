#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(s: &str) -> Self {
        Error {
            message: String::from(s),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Error"
    }
}

impl From<std::io::Error> for Error {
    fn from(_e: std::io::Error) -> Self {
        Error::new("IO Error")
    }
}

impl From<rusqlite::Error> for Error {
    fn from(_e: rusqlite::Error) -> Self {
        Error::new("Rusqlite Error")
    }
}

impl From<rexif::ExifError> for Error {
    fn from(_e: rexif::ExifError) -> Self {
        Error::new("Exif Error")
    }
}

impl From<image::ImageError> for Error {
    fn from(_e: image::ImageError) -> Self {
        Error::new("Image Error")
    }
}

impl From<&str> for Error {
    fn from(_e: &str) -> Self {
        Error::new(_e)
    }
}

impl From<std::ffi::OsString> for Error {
    fn from(_e: std::ffi::OsString) -> Self {
        Error::new("OS String Error")
    }
}

impl From<std::option::NoneError> for Error {
    fn from(_e: std::option::NoneError) -> Self {
        Error::new("None Error")
    }
}
