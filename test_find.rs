use std::path::Path;

fn main() {
    let dir = Path::new(r"C:\Users\hallo\.vx\store\msvc\14.42");
    let exe_name = "cl";
    
    let possible_names: Vec<String> = vec![
        format!("{}.exe", exe_name),
        format!("{}.cmd", exe_name),
        exe_name.to_string(),
    ];
    
    println!("Searching for {:?} in {}", possible_names, dir.display());
    
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if possible_names.iter().any(|n| n == name) {
                    println!("Found: {}", path.display());
                }
            }
        }
    }
}
