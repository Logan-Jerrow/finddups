use std::{
    cell::Cell,
    env,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

fn main() -> std::io::Result<()> {
    let args = env::args().skip(1);

    let (mut files, directories): (Vec<_>, Vec<_>) = args
        .filter_map(|ref name| match std::fs::symlink_metadata(name) {
            Ok(m) if m.is_file() | m.is_dir() => Some(PathBuf::from(name)),
            Ok(_) => None,
            Err(e) => {
                eprintln!("{e}  --  {name}");
                None
            }
        })
        .partition(|path| path.is_file());

    println!();

    for f in directories.iter() {
        let vis = visit_dirs(f, &mut |entry| {
            if entry.path().is_file() {
                files.push(entry.path());
            }
        });
    }
    let mut visited = vec![false; files.len()];

    for (v, f1) in files.iter().enumerate() {
        if *visited.get(v).expect("visited vector out of bounds!") {
            continue;
        }
        for (v2, f2) in files.iter().enumerate().skip(1) {
            let f1_size = f1.symlink_metadata().unwrap().len();
            let f2_size = f2.symlink_metadata().unwrap().len();
            if f1_size == f2_size {
                println!("{} and {} YES", f1.display(), f2.display());
                *visited
                    .get_mut(v2)
                    .expect("Visited vector inner loop out of bounds!") = true;
            }
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
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else if !path.is_symlink() {
                cb(&entry);
            }
        }
    }
    Ok(())
}

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
