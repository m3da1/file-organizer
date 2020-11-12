use std::{env, fs, io, path::Path, process::Command};

fn main() {
    let path = env::args().nth(1).expect("Directory is required");
    let list = dump_dir(path.as_str()).unwrap();
    move_files(list, path.as_str());
}

fn dump_dir(dir: &str) -> io::Result<Vec<(String, String)>> {
    let mut list: Vec<(String, String)> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let data = entry.metadata()?;
        let path = entry.path();
        if data.is_file() {
            let guess = mime_guess::from_path(&path);
            if let Some(v) = guess.first() {
                list.push((path.display().to_string(), v.to_string()));
            }
        }
    }
    Ok(list)
}

fn move_file(path: &str, name: &str, dest: &str) {
    let mut newpath = path.to_string();
    newpath.push_str("/");
    newpath.push_str(dest);
    let err = format!("Failed to chdir to {}", newpath);
    match check_directory(newpath.as_str()) {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to create directory: {}, Causes: {}", newpath, e);
            std::process::exit(1);
        }
    }
    std::env::set_current_dir(newpath).expect(err.as_str());
    let status = Command::new("mv")
        .args(&[name, "."])
        .status()
        .expect("failed to move file");
    println!("[StatusCode: {}] {}", status.success(), name);
    let err = format!("Failed to chdir to {}", path);
    std::env::set_current_dir(path).expect(err.as_str());
}

fn move_files(list: Vec<(String, String)>, path: &str) {
    for f in list {
        if f.1.contains("audio") || f.1.contains("video") || f.1.contains("image") {
            move_file(path, f.0.as_str(), "Multimedia");
        } else if f.1.contains("pdf") || f.1.contains("document") || f.1.contains("text") {
            move_file(path, f.0.as_str(), "Docs");
        } else if f.1.contains("zip") {
            move_file(path, f.0.as_str(), "Compressed")
        } else {
            move_file(path, f.0.as_str(), "Misc");
        }
    }
}

fn check_directory(d: &str) -> std::io::Result<()> {
    if !Path::new(d).is_dir() {
        fs::create_dir(d)?;
    }
    Ok(())
}
