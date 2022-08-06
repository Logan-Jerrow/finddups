use std::{
    env,
    fs::{self, DirEntry},
    io,
    path::Path,
};

use info::{Info, InfoKind};

mod info;

struct Group<'a> {
    count: usize,
    paths: Vec<&'a Info>,
}

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1);

    let (mut files, directories): (Vec<_>, Vec<_>) = args
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
