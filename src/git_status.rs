use std::fs;
use std::path::Path;
use std::process::Command;

use crate::git_database::GitRepoInfo;

pub fn is_git_repo(path: &Path) -> bool {
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

pub fn get_git_status(path: &Path) -> String {
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

pub fn get_unpushed_commits(path: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("rev-list")
        .arg("--branches")
        .arg("--not")
        .arg("--remotes")
        .output()
        .expect("Failed to execute git command")
        .stdout;

    String::from_utf8(output).unwrap()
}

pub fn get_remote_updates(path: &Path) -> String {
    Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("fetch")
        .output()
        .expect("Failed to execute git command");

    let output = Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("log")
        .arg("..origin/master")
        .arg("--oneline")
        .output()
        .expect("Failed to execute git command")
        .stdout;

    String::from_utf8(output).unwrap()
}

pub fn check_dir(path: &Path) -> Vec<GitRepoInfo> {
    let mut repos = Vec::new();

    if is_git_repo(path) {
        let status = get_git_status(path);
        let unpushed = get_unpushed_commits(path);
        let updates = get_remote_updates(path);

        Some(GitRepoInfo::new(
            path.to_str().unwrap().to_string(),
            status,
            unpushed,
            updates,
        ))
    } else if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            println!("entry: {:?}", &path);
            if path.is_dir() {
                repos.extend(check_dir(&path));
            }
        }
    }
    repos
}
