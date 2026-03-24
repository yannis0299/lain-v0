use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

// Translation Unit
#[derive(Debug, Clone)]
pub struct TU {
    pub filename: String,
    pub contents: String,
}

impl TU {
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<TU> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        let filename = path
            .as_ref()
            .file_name()
            .into_iter()
            .filter_map(|str| str.to_str())
            .next()
            .unwrap_or("<unknown>");
        file.read_to_string(&mut contents)?;
        Ok(Self {
            filename: filename.into(),
            contents,
        })
    }
}
