use std::{
    fs::File,
    io::{prelude::*, BufReader},
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
pub enum FileKind {
    Other,
    File,
    Directory,
    Link,
}

#[derive(Debug)]
pub struct FileData {
    pub path: PathBuf,
    pub size: u64,
    pub kind: FileKind,
}

impl FileData {
    pub fn new(path: PathBuf, size: u64, kind: FileKind) -> Self {
        Self { path, size, kind }
    }

    pub fn is_duplicate(&self, other: &Self) -> anyhow::Result<bool> {
        // Compare file sizes.
        if self.size != other.size {
            return Ok(false);
        }

        let file1 = File::open(&self.path)?;
        let file1 = BufReader::new(file1);

        let file2 = File::open(&other.path)?;
        let file2 = BufReader::new(file2);

        // byte by byte comparasion.
        for (b1, b2) in file1.bytes().zip(file2.bytes()) {
            if b1? != b2? {
                return Ok(false);
            }
        }

        Ok(true)
    }
    pub fn from_metadata(path: impl Into<PathBuf>, meta: std::fs::Metadata) -> Self {
        let kind = match &meta {
            m if m.is_file() => FileKind::File,
            m if m.is_dir() => FileKind::Directory,
            m if m.is_symlink() => FileKind::Link,
            _ => FileKind::Other,
        };
        FileData::new(path.into(), meta.len(), kind)
    }
}
