use file_data::{FileData, FileKind};
use std::{env, path::Path};
use walkdir::WalkDir;

mod file_data;

#[derive(Debug)]
struct Group {
    count: usize,
    paths: Vec<FileData>,
}

impl Group {
    fn new(count: usize, paths: Vec<FileData>) -> Self {
        Group { count, paths }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self {
            count: 1,
            paths: Default::default(),
        }
    }
}

fn get_args() -> Vec<String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        // If given no args then add the local folder "."
        args.push(".".to_string());
    }
    args
}

fn main() -> anyhow::Result<()> {
    let args = get_args();

    let files = get_files(args);
    assert!(files.iter().all(|f| f.path.is_file()));

    let (groups, errors) = get_groups(files);

    for g in groups {
        for (i, p) in g.paths.into_iter().enumerate() {
            println!("{} {} {}", g.count, i + 1, p.path.display());
        }
    }

    if !errors.is_empty() {
        eprintln!("\n\nErrors!!!: {:#?}", errors);
    }

    Ok(())
}

fn transverse(dir: &Path, files: &mut Vec<FileData>) {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .for_each(|entry| match entry.metadata() {
            Ok(meta) => {
                let fd = FileData::from_metadata(entry.into_path(), meta);
                files.push(fd);
            }
            Err(e) => {
                // TODO: Return a list of errors?
                eprintln!("transverse cannot read metadata. {e}");
            }
        });
}

fn get_files(args: Vec<String>) -> Vec<FileData> {
    // TODO: Add files from local env if no args given.
    // List files and directories from working directory.
    let (mut files, directories): (Vec<_>, Vec<_>) = args
        .into_iter()
        .filter_map(|name| match std::fs::symlink_metadata(&name) {
            Ok(meta) => Some(FileData::from_metadata(name, meta)),
            Err(e) => {
                eprintln!("symlink metadata failed on path: \"{name}\". {e}");
                None
            }
        })
        .partition(|fd| fd.kind == FileKind::File);

    // Transverse directories grabbing every file path.
    for f in directories.iter() {
        transverse(&f.path, &mut files);
    }

    files
}

fn get_groups(mut files: Vec<FileData>) -> (Vec<Group>, Vec<anyhow::Error>) {
    let mut errors = vec![];
    let mut groups: Vec<Group> = Vec::with_capacity(files.len());

    while let Some(f) = files.pop() {
        // Partition(split) duplicate files(f).
        let (mut dups, leftover): (Vec<FileData>, Vec<FileData>) =
            files.into_iter().partition(|d| match f.is_duplicate(d) {
                Ok(pred) => pred,
                Err(e) => {
                    errors.push(e);
                    false
                }
            });
        files = leftover;

        dups.push(f);
        dups.sort_unstable_by(|a, b| a.path.as_os_str().len().cmp(&b.path.as_os_str().len()));

        groups.push(Group::new(dups.len(), dups));
    }

    groups.sort_unstable_by(|a, b| a.count.cmp(&b.count));
    (groups, errors)
}
