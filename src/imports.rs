use std::fs;
use std::fs::DirEntry;
use std::io::Result;

pub fn directory_to_files(path: &str, extensions: &Vec<&str>) -> Vec<Result<DirEntry>> {
    let paths = fs::read_dir(path).unwrap();
    let filtered_files: Vec<_> = paths
        .into_iter()
        .filter(|path| file_extension_filter(path, extensions))
        .collect();
    return filtered_files;
}

pub fn file_extension_filter(path: &Result<DirEntry>, extensions: &Vec<&str>) -> bool {
    let file_name = path.as_ref().unwrap().file_name().into_string().unwrap();
    let file_extension = file_name.split(".").last().unwrap();
    return extensions.contains(&file_extension);
}