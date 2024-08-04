use log::debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
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

pub fn get_remote_origin(path: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .output()
        .expect("Failed to execute git command")
        .stdout;

    String::from_utf8(output).unwrap().trim().to_string()
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

    debug!("Checking path: {:?}", &path);

    if is_git_repo(path) {
        let status = get_git_status(path);
        let unpushed = get_unpushed_commits(path);
        let updates = get_remote_updates(path);
        let origin_url = get_remote_origin(path);

        let repo_info = GitRepoInfo::new(
            path.to_str().unwrap().to_string(),
            Some(origin_url),
            status,
            unpushed,
            updates,
        );

        repos.push(repo_info);
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

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    use std::path::Path;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_is_git_repo() {
        let path = Path::new("/home/sinh/Downloads/");
        assert!(!is_git_repo(&path));

        let path_git = Path::new("/home/sinh/git-repos/sinh-x/sinh-x-gitstatus/");
        assert!(is_git_repo(&path_git));
    }

    #[test]
    fn test_get_git_status() {
        let path = Path::new("path_to_test_repo");
        let status = get_git_status(&path);

        todo!("Test not implemented yet");
        // assert something about the status
    }

    #[test]
    fn test_get_unpushed_commits() {
        let path = Path::new("path_to_test_repo");
        let unpushed = get_unpushed_commits(&path);
        // assert something about the unpushed commits
        todo!("Test not implemented yet");
    }

    #[test]
    fn test_get_remote_updates() {
        let path = Path::new("path_to_test_repo");
        let updates = get_remote_updates(&path);
        // assert something about the remote updates
        todo!("Test not implemented yet");
    }

    #[test]
    fn test_check_dir() {
        init();

        // assert something about the repos
        let path_with_subdir = Path::new("/home/sinh/git-repos/andafin");
        let dir_repos_info = check_dir(&path_with_subdir);
        assert!(
            dir_repos_info.len() > 1,
            "Expected 1 repo, but found {} repos",
            dir_repos_info.len()
        );
    }
}
