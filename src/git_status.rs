use crate::git_database::GitRepoInfo;
use log::debug;
use semver::Version;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

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
            Some(Version::parse(env!("CARGO_PKG_VERSION")).unwrap()),
        );

        repos.push(repo_info);
    } else if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            debug!("entry: {:?}", &path);
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
    fn test_gitstatus_functions() {
        init();

        // assert something about the repos
        let path_with_subdir = Path::new("/home/sinh/git-repos/others/rust-analyzer/");
        let dir_repos_info = check_dir(&path_with_subdir);
        assert!(
            dir_repos_info.len() == 1,
            "Expected 1 repo, but found {} repos",
            dir_repos_info.len()
        );
        let repo = &dir_repos_info[0];
        let remote_updates_count = repo.remote_updates.matches('\n').count();
        let unpushed_commits_count = repo.unpushed_commits.matches('\n').count();
        let changes_count = repo.status.matches('\n').count();

        assert!(
            changes_count == 5,
            "Expected 5 changes_count found {}",
            changes_count,
        );
        assert!(
            remote_updates_count >= 523,
            "Expected at least one remote update found {}",
            remote_updates_count,
        );
        assert!(
            unpushed_commits_count == 1,
            "Expected 1 unpushed_commits found {}",
            unpushed_commits_count
        );
    }

    #[test]
    fn test_check_dir() {
        init();

        // assert something about the repos
        let path_with_subdir = Path::new("/home/sinh/git-repos/andafin/old-projects/");
        let dir_repos_info = check_dir(&path_with_subdir);
        assert!(
            dir_repos_info.len() == 6,
            "Expected 6 repo, but found {} repos",
            dir_repos_info.len()
        );
    }
}
