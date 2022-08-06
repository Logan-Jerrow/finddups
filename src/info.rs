use std::{
    fs::{DirEntry, File, Metadata},
    io::{self, prelude::*, BufReader},
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
pub enum InfoKind {
    Other,
    File,
    Directory,
    Link,
}
pub struct Info {
    pub path: PathBuf,
    pub size: u64,
    pub kind: InfoKind,
}

impl Info {
    pub fn new(path: PathBuf, size: u64, kind: InfoKind) -> Self {
        Self { path, size, kind }
    }

    pub fn is_file(&self) -> bool {
        self.kind == InfoKind::File
    }

    pub fn is_dir(&self) -> bool {
        self.kind == InfoKind::Directory
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
            // TODO: Replace unwrap.
            if b1? != b2? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn from_metadata<P: AsRef<Path>>(path: P, meta: Metadata) -> Self {
        Info::from_internal(path.as_ref().to_path_buf(), meta)
    }

    fn from_internal(path: PathBuf, meta: Metadata) -> Self {
        match meta {
            m if m.is_file() => Info::new(path, m.size(), InfoKind::File),
            m if m.is_dir() => Info::new(path, m.size(), InfoKind::Directory),
            m if m.is_symlink() => Info::new(path, m.size(), InfoKind::Link),
            m => Info::new(path, m.size(), InfoKind::Other),
        }
    }
}

impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        if self.size == other.size {
            let file1 = File::open(&self.path);
            let file2 = File::open(&other.path);
            if file1.is_err() {
                eprintln!("{}\t{}", self.path.display(), file1.as_ref().unwrap_err());
            }
            if file2.is_err() {
                eprintln!("{}\t{}", other.path.display(), file2.as_ref().unwrap_err());
            }

            let file1 = file1.unwrap();
            let file2 = file2.unwrap();

            let mut buf1 = BufReader::new(file1);
            let mut buf2 = BufReader::new(file2);

            let mut contents1 = String::new();
            let mut contents2 = String::new();

            if let Err(e) = buf1.read_to_string(&mut contents1) {
                eprintln!("{}\t{}", self.path.display(), e);
                return false;
            }
            if let Err(e) = buf2.read_to_string(&mut contents2) {
                eprintln!("{}\t{}", other.path.display(), e);
                return false;
            }

            if contents1 == contents2 {
                return true;
            }
        }
        false
    }
}

impl Eq for Info {}

impl TryFrom<String> for Info {
    type Error = std::io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match std::fs::symlink_metadata(&value) {
            Ok(m) => Ok(Info::from_metadata(value, m)),
            Err(e) => {
                eprintln!("{e}  --  {value}");
                Err(e)
            }
        }
    }
}

impl TryFrom<&DirEntry> for Info {
    type Error = io::Error;

    fn try_from(value: &DirEntry) -> Result<Self, Self::Error> {
        match value.metadata() {
            Ok(m) => Ok(Info::from_metadata(value.path(), m)),
            Err(e) => {
                eprintln!("{e}  --  {:#?}", value);
                Err(e)
            }
        }
    }
}
