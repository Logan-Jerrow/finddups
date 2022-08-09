use crate::FileData;

#[derive(Debug)]
pub struct Group {
    pub count: usize,
    pub paths: Vec<FileData>,
}

impl Group {
    pub fn new(count: usize, paths: Vec<FileData>) -> Self {
        Group { count, paths }
    }
}
