use std::{
    env,
    fs::{self, DirEntry, File, Metadata},
    io::{self, prelude::*, BufReader},
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
enum InfoKind {
    Other,
    File,
    Directory,
    Link,
}
struct Info {
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

    pub fn from_metadata<P: AsRef<Path>>(path: P, meta: Metadata) -> Self {
        Info::from_internal(path.as_ref().to_path_buf(), meta)
    }

    fn from_internal(path: PathBuf, meta: Metadata) -> Self {
        match meta {
            m if m.is_file() => Info::new(path, m.size(), InfoKind::File),
            m if m.is_dir() => Info::new(path, m.size(), InfoKind::Directory),
            m if m.is_symlink() => Info::new(path, m.size(), InfoKind::File),
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
            }
            if let Err(e) = buf2.read_to_string(&mut contents2) {
                eprintln!("{}\t{}", other.path.display(), e);
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
                eprintln!("{e}  --  {}");
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
                eprintln!("{e}  --  {}");
                Err(e)
            }
        }
    }
}

struct Group<'a> {
    count: usize,
    paths: Vec<&'a Info>,
}

fn main() -> std::io::Result<()> {
    let args = env::args().skip(1);

    let (mut files, directories): (Vec<_>, Vec<_>) = args
        // .filter_map(|ref name| match std::fs::symlink_metadata(name) {
        //     Ok(m) if m.is_file() | m.is_dir() => Some(PathBuf::from(name)),
        //     Ok(_) => None,
        //     Err(e) => {
        //         eprintln!("{e}  --  {name}");
        //         None
        //     }
        // })
        .filter_map(|name| match Info::try_from(name) {
            Ok(info) if info.is_file_or_dir() => Some(info),
            Ok(_) => None,
            Err(_) => None,
        })
        .partition(|info| info.kind == InfoKind::File);

    println!();

    for f in directories.iter() {
        let visit = visit_dirs(&f.path, &mut |entry| {
            if let Ok(info) = Info::try_from(entry) {
                files.push(info);
            };
        });

        if visit.is_err() {
            eprintln!("Error visiting directories: {}", visit.unwrap_err());
        }
    }
    let mut visited = vec![false; files.len()];

    let mut groups: Vec<Group> = Vec::new();
    for (index_first, f1) in files.iter().enumerate() {
        if *visited
            .get(index_first)
            .expect("visited vector out of bounds!")
        {
            continue;
        }
        let mut group = Group {
            count: 1,
            paths: Vec::new(),
        };
        group.paths.push(f1);
        for (index_second, f2) in files.iter().enumerate().skip(index_first + 1) {
            if f1 == f2 {
                // println!("{} and {} YES", f1.path.display(), f2.path.display());
                group.count += 1;
                group.paths.push(f2);
                *visited
                    .get_mut(index_second)
                    .expect("Visited vector inner loop out of bounds!") = true;
            }
        }
        if group.count > 1 {
            groups.push(group);
        }
    }

    for g in groups {
        for (i, p) in g.paths.into_iter().enumerate() {
            println!("{} {} {}", g.count, i + 1, p.path.display());
        }
    }

    // let visited = vec![Cell::new(false); files.len()];
    // let visited = visited.into_iter().zip(files).collect::<Vec<(_, _)>>();

    // for (v, f1) in visited.iter() {
    //     if v.get() {
    //         println!("Already visited: {}", f1.display());
    //         continue;
    //     }
    //     for (v2, f2) in visited.iter().skip(1) {
    //         let f1_size = f1.symlink_metadata().unwrap().len();
    //         let f2_size = f2.symlink_metadata().unwrap().len();
    //         if f1_size == f2_size {
    //             println!("{} and {} are the same size.", f1.display(), f2.display());
    //         } else {
    //             println!("{} and {} NO.", f1.display(), f2.display());
    //         }
    //         v2.set(true);
    //     }
    // }

    Ok(())
}

/// One possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            // let path = entry.path();
            // if path.is_dir() {
            if ft.is_dir() {
                visit_dirs(&entry.path(), cb)?;
            } else if ft.is_file() {
                cb(&entry);
            }
        }
    }
    Ok(())
}

#[allow(unused)]
mod info {
    use std::convert::TryFrom;
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct Info {
        pub path: PathBuf,
        // file_type: FileType,
        meta: fs::Metadata,
    }

    impl Info {
        pub fn new<P>(path: P, meta: std::fs::Metadata) -> Info
        where
            P: AsRef<Path>,
        {
            Info {
                path: path.as_ref().to_path_buf(),
                meta,
            }
        }

        pub fn is_file(&self) -> bool {
            self.meta.is_file()
        }

        pub fn is_dir(&self) -> bool {
            self.meta.is_dir()
        }

        pub fn is_other(&self) -> bool {
            !(self.is_file() || self.is_dir())
        }
    }

    /// [`FileType`] Different types a [`Info`] can be.
    ///
    /// [`FileType`]: FileType
    /// [`Info`]: Info
    // #[derive(Debug)]
    // pub enum FileType {
    //     File,
    //     Directory,
    //     Link,
    //     Other,
    // }

    // impl FileType {
    //     /// Returns `true` if the file type is [`File`].
    //     ///
    //     /// [`File`]: FileType::File
    //     pub fn is_file(&self) -> bool {
    //         matches!(self, Self::File)
    //     }

    //     /// Returns `true` if the file type is [`Directory`].
    //     ///
    //     /// [`Directory`]: FileType::Directory
    //     pub fn is_directory(&self) -> bool {
    //         matches!(self, Self::Directory)
    //     }

    //     /// Returns `true` if the file type is [`File`] or [`Directory`].
    //     ///
    //     /// [`File`]: FileType::File
    //     /// [`Directory`]: FileType::Directory
    //     pub fn is_file_or_dir(&self) -> bool {
    //         matches!(self, Self::File | Self::Directory)
    //     }
    // }

    // impl Default for FileType {
    //     fn default() -> Self {
    //         Self::Other
    //     }
    // }

    impl TryFrom<String> for Info {
        type Error = error::InfoError;
        fn try_from(path: String) -> Result<Self, Self::Error> {
            match fs::symlink_metadata(&path) {
                Ok(meta) => Ok(Info::new(path, meta)),
                Err(error) => Err(error::InfoError::FileNotFound {
                    source: error,
                    path,
                }),
            }
        }
    }

    pub mod error {
        #[derive(Debug)]
        pub enum InfoError {
            FileNotFound {
                source: std::io::Error,
                path: String,
            },
        }

        impl std::error::Error for InfoError {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match *self {
                    InfoError::FileNotFound { ref source, .. } => Some(source),
                }
            }
        }

        impl std::fmt::Display for InfoError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    InfoError::FileNotFound {
                        ref source,
                        ref path,
                    } => {
                        write!(f, "{source} Path: \"{path}\"",)
                    }
                }
            }
        }
    }
}
