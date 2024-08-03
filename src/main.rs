use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .expect("Failed to execute git command")
        .status
        .success()
}

fn get_git_status(path: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("Failed to execute git command")
        .stdout;

    String::from_utf8(output).unwrap()
}

fn check_dir(path: &Path) {
    if is_git_repo(path) {
        let status = get_git_status(path);
        if !status.is_empty() {
            println!("{} has changes:\n{}", path.display(), status);
        }
    } else if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a list of directories to check.");
        return;
    }

    for path_str in &args[1..] {
        let path = Path::new(path_str);
        check_dir(path);
    }
}
