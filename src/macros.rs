use std::fs;

#[macro_export]
macro_rules! collect_files {
    ($dir:expr, $($ext:expr),+) => {{
        use std::fs;
        use std::path::Path;

        let dir_path = Path::new($dir);

        let files: Vec<String> = fs::read_dir(dir_path)
            .expect("Failed to read directory!")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().ok().map(|ft| ft.is_file()).unwrap_or(false))
            .filter_map(|entry| {
                let ext = entry.path().extension().map(|ext| ext.to_string_lossy().to_lowercase());
                match ext {
                    $(Some(ext) if ext == $ext.to_string().to_lowercase() => Some(entry.path().to_string_lossy().to_string()),)*
                    _ => None,
                }
            })
            .collect();

        if files.is_empty() {
            Vec::new()
        } else {
            files
        }
    }};
}

pub fn find_subfolder(path: &str, pattern: &str) -> Option<String> {
    // find the subfolder that contains the pattern
    fs::read_dir(path)
        .ok()?
        .find_map(|entry| entry.ok().filter(|e| e.file_type().ok().map_or(false, |ft| ft.is_dir())).map(|e| e.path().to_string_lossy().to_string()))
        .and_then(|subfolder| {
            if subfolder.contains(pattern) {
                Some(subfolder)
            } else {
                find_subfolder(&subfolder, pattern)
            }
        })
}