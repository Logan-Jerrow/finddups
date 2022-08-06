use std::{
    fs::{DirEntry, File},
    io::{prelude::*, BufReader},
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
pub enum FileKind {
    Other,
    File,
    Directory,
    Link,
}
pub struct Duplicate {
    pub path: PathBuf,
    pub size: u64,
    pub kind: FileKind,
}

impl Duplicate {
    pub fn new(path: PathBuf, size: u64, kind: FileKind) -> Self {
        Self { path, size, kind }
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileKind::File
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FileKind::Directory
    }

    pub fn is_file_or_dir(&self) -> bool {
        self.is_file() || self.is_dir()
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

    pub fn from_path<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let meta = std::fs::symlink_metadata(&path)?;

        let fk = match &meta {
            m if m.is_file() => FileKind::File,
            m if m.is_dir() => FileKind::Directory,
            m if m.is_symlink() => FileKind::Link,
            _ => FileKind::Other,
        };
        Ok(Duplicate::new(path.as_ref().to_path_buf(), meta.size(), fk))
    }
}

impl TryFrom<String> for Duplicate {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Duplicate::from_path(value)
    }
}

impl TryFrom<&DirEntry> for Duplicate {
    type Error = anyhow::Error;

    fn try_from(value: &DirEntry) -> Result<Self, Self::Error> {
        Duplicate::from_path(value.path())
    }
}
