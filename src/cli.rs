use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use structopt::StructOpt;

/// This application organizes the folder into categories (eg: Docs, Multimedia etc)
#[derive(Debug, StructOpt)]
pub struct MyOrganizer {
    /// Path to organize
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,
}

pub fn organizer_files(path: PathBuf) -> std::io::Result<()> {
    println!("----[ Organizing ({}) in Rust ]----", &path.display());
    let list = dump_dir(&path)?;
    move_files(list, path.into_os_string().into_string().unwrap().as_str());
    Ok(())
}

fn dump_dir(dir: &PathBuf) -> io::Result<Vec<(String, String)>> {
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
    newpath.push_str(std::path::MAIN_SEPARATOR.to_string().as_str());
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
    let cmd = if cfg!(target_os = "windows") {
        "move"
    } else {
        "mv"
    };
    let status = Command::new(cmd)
        .args(&[name, "."])
        .status()
        .expect("failed to move file");
    println!(" [StatusCode: {}]", status.success());
    let err = format!("Failed to chdir to {}", path);
    std::env::set_current_dir(path).expect(err.as_str());
}

fn move_files(list: Vec<(String, String)>, path: &str) {
    for f in list {
        print!("file: [{}] type: [{}]", f.0, f.1);
        match f.1.as_str() {
            "image/png" | "audio/mpeg" | "image/jpeg" | "audio/ogg" => {
                move_file(path, f.0.as_str(), "Multimedia")
            }
            "application/zip" => move_file(path, f.0.as_str(), "Compressed"),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/pdf"
            | "text/html"
            | "text/csv"
            | "text/xml" => {
                move_file(path, f.0.as_str(), "Docs");
            }
            _ => move_file(path, f.0.as_str(), "Misc"),
        }
    }
}

fn check_directory(d: &str) -> std::io::Result<()> {
    if !Path::new(d).is_dir() {
        fs::create_dir(d)?;
    }
    Ok(())
}
