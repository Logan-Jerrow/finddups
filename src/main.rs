use std::{
    env,
    fs::{self, DirEntry},
    io,
    path::Path,
};

use file_data::{FileData, FileKind};
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

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1);

    let files = get_files(args);

    let (groups, errors) = get_groups(files);

    for g in groups {
        for (i, p) in g.paths.into_iter().enumerate() {
            println!("{} {} {}", g.count, i + 1, p.path.display());
        }
    }

    if !errors.is_empty() {
        eprintln!("\n\nErrors: {:#?}", errors);
    }

    Ok(())
}

/// One possible implementation of walking a directory only visiting files
fn visit_dirs<C>(dir: &Path, cb: &mut C) -> io::Result<()>
where
    C: FnMut(&DirEntry),
{
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)?.filter_map(Result::ok) {
        if let Ok(ft) = entry.file_type() {
            if ft.is_dir() {
                visit_dirs(&entry.path(), cb)?;
            } else if ft.is_file() {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn transverse<C>(dir: &Path, files: &mut Vec<FileData>) -> anyhow::Result<()> {
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let fd = FileData::from_direntry(entry)?;
        files.push(fd);
    }
    Ok(())
}

fn get_files(args: impl Iterator<Item = String>) -> Vec<FileData> {
    // TODO: Add files from local env if no args given.
    // List files and directories from working directory.
    let (mut files, directories): (Vec<_>, Vec<_>) = args
        .filter_map(|name| match FileData::from_path(name) {
            Ok(info) if info.is_file_or_dir() => Some(info),
            Ok(_) => None,
            Err(_) => None,
        })
        .partition(|info| info.kind == FileKind::File);

    // Transverse directories grabbing every file path.
    for f in directories.iter() {
        let visit = visit_dirs(&f.path, &mut |entry| {
            if let Ok(info) = FileData::from_path(entry.path()) {
                files.push(info);
            };
        });

        if visit.is_err() {
            eprintln!("Error visiting directories: {}", visit.unwrap_err());
        }
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
