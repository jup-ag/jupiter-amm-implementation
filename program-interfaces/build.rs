use std::path::Path;

fn main() {
    for file in find_files(Path::new("../idls")) {
        println!("cargo:rerun-if-changed={file}");
    }
}

fn find_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_dir() {
            // Add file paths as strings
            if let Some(path_str) = path.to_str() {
                files.push(path_str.to_string());
            }
        }
    }

    files
}
