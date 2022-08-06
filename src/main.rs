use std::{
    env,
    fs::{self, DirEntry},
    io,
    path::Path,
};

use info::{Duplicate, FileKind};

mod info;

struct Group<'a> {
    count: usize,
    paths: Vec<&'a Duplicate>,
}

impl<'a> Default for Group<'a> {
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

    let groups = get_groups(&files)?;

    for g in groups {
        for (i, p) in g.paths.into_iter().enumerate() {
            println!("{} {} {}", g.count, i + 1, p.path.display());
        }
    }

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

fn get_files(args: impl Iterator<Item = String>) -> Vec<Duplicate> {
    // TODO: Add files from local env if no args given.
    // List files and directories from working directory.
    let (mut files, directories): (Vec<_>, Vec<_>) = args
        .filter_map(|name| match Duplicate::try_from(name) {
            Ok(info) if info.is_file_or_dir() => Some(info),
            Ok(_) => None,
            Err(_) => None,
        })
        .partition(|info| info.kind == FileKind::File);

    // Transverse directories grabbing every file path.
    for f in directories.iter() {
        let visit = visit_dirs(&f.path, &mut |entry| {
            if let Ok(info) = Duplicate::try_from(entry) {
                files.push(info);
            };
        });

        if visit.is_err() {
            eprintln!("Error visiting directories: {}", visit.unwrap_err());
        }
    }

    files
}

fn get_groups(files: &[Duplicate]) -> anyhow::Result<Vec<Group>> {
    let mut visited = vec![false; files.len()];
    let mut groups: Vec<Group> = Vec::with_capacity(files.len());

    for (i, f1) in files.iter().enumerate() {
        if *visited.get(i).expect("visited vector out of bounds!") {
            continue;
        }
        let mut group = Group::default();

        group.paths.push(f1);
        for (j, f2) in files.iter().enumerate() {
            // If same file don't compare.
            if i == j {
                break;
            }
            if f1.is_duplicate(f2)? {
                group.count += 1;
                group.paths.push(f2);
                *visited
                    .get_mut(j)
                    .expect("Visited vector inner loop out of bounds!") = true;
            }
        }
        if group.count > 1 {
            groups.push(group);
        }
    }
    Ok(groups)
}
