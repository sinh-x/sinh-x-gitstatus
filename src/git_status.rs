use crate::git_database::{
    GitCommit, GitDatabase, GitDatabaseError, GitRepoInfo, SerializableTime,
};
use colored::Colorize;
use git2::{Cred, RemoteCallbacks, Repository};
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use log::warn;
use semver::Version;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tokei::{Config as TokeiCfg, Languages};

#[derive(Debug)]
pub enum GitStatusError {
    InvalidDetailLevel,
    Git2(git2::Error),
    NoGitRepoFound,
    GitDatabaseError(crate::git_database::GitDatabaseError),
    // Add more variants for errors specific to module 1
}

impl std::fmt::Display for GitStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GitStatusError::InvalidDetailLevel => {
                write!(f, "GitStatus:: Details level must be between 0, 1")
            }
            GitStatusError::NoGitRepoFound => write!(f, "GitStatus:: No git repo found"),
            GitStatusError::Git2(err) => write!(f, "GitStatus:: git error: {}", err),
            GitStatusError::GitDatabaseError(err) => {
                write!(f, "GitStatus:: database access error: {}", err)
            }
        }
    }
}

impl From<git2::Error> for GitStatusError {
    fn from(err: git2::Error) -> GitStatusError {
        GitStatusError::Git2(err)
    }
}

impl From<GitDatabaseError> for GitStatusError {
    fn from(err: GitDatabaseError) -> GitStatusError {
        GitStatusError::GitDatabaseError(err)
    }
}

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
    debug!("get_remote_origin: {}", path.display());
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
    debug!("get_git_status: {}", path.display());

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
    debug!("get_unpushed_commits: {}", path.display());
    let remote_url_output = Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .expect("Failed to execute git command");

    let repo = Repository::open(path).expect("Failed to open repository");
    let remote_url = String::from_utf8(remote_url_output.stdout).unwrap();
    if remote_url.starts_with("http") {
        let mut remote = repo.find_remote("origin").expect("Failed to find remote");
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| Cred::default());

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Attempt to fetch from the remote repository
        match remote.fetch(
            &["refs/heads/*:refs/heads/*"],
            Some(&mut fetch_options),
            None,
        ) {
            Ok(_) => debug!(
                "The repository at {} does not require authentication.",
                remote_url
            ),
            Err(e) => {
                if e.message().contains("authentication required") {
                    warn!(
                        "get_remote_updates: http protocol with password authentication is not support! {}",
                        remote_url
                    );
                } else {
                    println!(
                        "get_remote_updates: Failed to fetch from {}: {}",
                        remote_url, e
                    );
                }
                return String::new();
            }
        }
    }

    debug!(
        "get_unpushed_commits: checking http passed {}",
        path.display()
    );
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
    debug!("get_remote_updates: {}", path.display());
    // Get the URL of the remote origin
    let remote_url_output = Command::new("git")
        .arg("-C")
        .arg(path.to_str().unwrap())
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .expect("Failed to execute git command");

    let repo = Repository::open(path).expect("Failed to open repository");
    let remote_url = String::from_utf8(remote_url_output.stdout).unwrap();
    if remote_url.starts_with("http") {
        let mut remote = repo.find_remote("origin").expect("Failed to find remote");
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| Cred::default());

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Attempt to fetch from the remote repository
        match remote.fetch(
            &["refs/heads/*:refs/heads/*"],
            Some(&mut fetch_options),
            None,
        ) {
            Ok(_) => debug!(
                "The repository at {} does not require authentication.",
                remote_url
            ),
            Err(e) => {
                if e.message().contains("authentication required") {
                    warn!(
                        "get_remote_updates: http protocol with password authentication is not support! {}",
                        remote_url
                    );
                } else {
                    println!(
                        "get_remote_updates: Failed to fetch from {}: {}",
                        remote_url, e
                    );
                }
                return String::new();
            }
        }
    }

    // Proceed with the original logic
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

fn check_git_paths(path: &Path) -> Result<Vec<PathBuf>, GitStatusError> {
    let mut git_paths = Vec::new();
    if is_git_repo(path) {
        git_paths.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                match check_git_paths(&path) {
                    Ok(mut paths) => git_paths.append(&mut paths),
                    Err(e) => {
                        debug!("Error processing path {}: {}", path.display(), e);
                        continue;
                    }
                }
            }
        }
    }

    if git_paths.is_empty() {
        return Err(GitStatusError::NoGitRepoFound);
    } else {
        Ok(git_paths)
    }
}

pub async fn check_dir(
    path: &Path,
    detail_level: &u8,
    gitdb: &GitDatabase,
) -> Result<Vec<GitRepoInfo>, GitStatusError> {
    let mut repos = Vec::new();

    debug!("Checking path: {:?}", &path);
    let git_paths = match check_git_paths(path) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error processing path {}: {}", path.display(), e);
            return Err(e);
        }
    };

    let pb = ProgressBar::new(git_paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("Failed to create progress bar style"),
    );
    pb.tick(); // Redraw the progress bar immediately
               //
    let tasks: Vec<_> = git_paths
        .into_iter()
        .map(|repo| {
            let pb = pb.clone();
            let detail_level = *detail_level;
            let gitdb = gitdb.clone();
            tokio::spawn(async move {
                let start = std::time::Instant::now();

                let status = get_git_status(&repo);
                let unpushed = get_unpushed_commits(&repo);
                let updates = get_remote_updates(&repo);
                let origin_url = get_remote_origin(&repo);
                let languages = match get_languages_summary(&repo, &detail_level, &gitdb) {
                    Ok(languages) => languages,
                    Err(e) => {
                        println!(
                            "Repo not existed in DB return an empty Languages. Error::{}",
                            e
                        );
                        Languages::new()
                    }
                };
                let commits_list = match get_commits_history(&repo, &detail_level, &gitdb) {
                    Ok(commits) => commits,
                    Err(e) => {
                        debug!("Repo not existed in DB return an empty Vec. Error::{}", e);
                        Vec::new()
                    }
                };

                let repo_info = GitRepoInfo::new(
                    repo.to_str().unwrap().to_string(),
                    Some(origin_url),
                    status,
                    unpushed,
                    updates,
                    Some(Version::parse(env!("CARGO_PKG_VERSION")).unwrap()),
                    Some(commits_list),
                    Some(languages),
                );

                pb.inc(1);

                let duration = start.elapsed();
                Ok::<_, GitStatusError>((repo_info, duration))
            })
        })
        .collect();

    let results: Result<Vec<_>, _> = futures::future::try_join_all(tasks).await;

    let mut durations = Vec::new();

    match results {
        Ok(repo_infos) => {
            for repo_info in repo_infos {
                let (repo_info, duration) = repo_info?;
                durations.push((repo_info.path.clone(), duration));
                repos.push(repo_info);
            }
        }
        Err(e) => eprintln!("Error spawning task: {}", e),
    }

    pb.finish_with_message("done");

    // Sort durations by descending order and take the first 3
    durations.sort_by(|a, b| b.1.cmp(&a.1));
    let longest_durations = durations.into_iter().take(3);

    for (path, duration) in longest_durations {
        debug!("Path: {}, Duration: {:?}", path, duration);
    }

    Ok(repos)
}

fn get_languages_summary(
    path: &Path,
    detail_level: &u8,
    gitdb: &GitDatabase,
) -> Result<Languages, GitStatusError> {
    let languages = match detail_level {
        &0 => match gitdb.get_repo_details(path.to_path_buf()) {
            Ok(repo) => {
                let required_version = Version::parse("0.6.0").unwrap();
                if repo.app_version >= required_version {
                    match repo.languages {
                        Some(languages) => languages,
                        None => Languages::new(),
                    }
                } else {
                    debug!(
                        "{}",
                        format!("WARNING: Old version of data {}. Please run with Check command to update the repo!", repo.app_version).yellow());
                    Languages::new()
                }
            }
            Err(e) => return Err(GitStatusError::GitDatabaseError(e)),
        },
        &1 => {
            let config = TokeiCfg::default();
            let mut languages = Languages::new();
            languages.get_statistics(&[path], &[], &config);
            languages
        }

        _ => return Err(GitStatusError::InvalidDetailLevel),
    };
    Ok(languages)
}

fn get_commits_history(
    path: &Path,
    detail_level: &u8,
    gitdb: &GitDatabase,
) -> Result<Vec<GitCommit>, GitStatusError> {
    let mut commits_list = Vec::new();
    match detail_level {
        &0 => match gitdb.get_repo_details(path.to_path_buf()) {
            Ok(repo) => {
                return Ok(repo
                    .commits
                    .expect("GitStatus:: failed to get commits history form SledDB"))
            }
            Err(e) => return Err(GitStatusError::GitDatabaseError(e)),
        },
        &1 => {
            let repo = Repository::open(path)?;
            let mut revwalk = repo.revwalk()?;
            revwalk.push_head()?;
            for id in revwalk {
                let id = id?;
                let commit = repo.find_commit(id)?;
                let diff = if commit.parent_count() > 0 {
                    let parent = commit.parent(0)?;
                    repo.diff_tree_to_tree(Some(&parent.tree()?), Some(&commit.tree()?), None)?
                } else {
                    let tree = commit.tree()?;
                    let empty_tree = repo.treebuilder(None)?.write()?;
                    let empty_tree = repo.find_tree(empty_tree)?;
                    repo.diff_tree_to_tree(Some(&empty_tree), Some(&tree), None)?
                };
                let stats = diff.stats()?;
                let author_email = commit.author().email().unwrap_or("").to_string();
                let message = commit.message().unwrap_or("").to_string();
                let git_commits = GitCommit::new(
                    id.to_string(),
                    author_email,
                    SerializableTime(commit.time()),
                    message,
                    stats.files_changed(),
                    stats.insertions(),
                    stats.deletions(),
                );
                commits_list.push(git_commits)
            }
        }
        _ => return Err(GitStatusError::InvalidDetailLevel),
    }
    Ok(commits_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    use std::path::Path;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[cfg(feature = "dev")]
    #[test]
    fn test_is_git_repo() {
        let path = Path::new("/home/sinh/Downloads/");
        assert!(!is_git_repo(&path));

        let path_git = Path::new("/home/sinh/git-repos/sinh-x/sinh-x-gitstatus/");
        assert!(is_git_repo(&path_git));
    }

    #[cfg(feature = "dev")]
    #[tokio::test]
    async fn test_gitstatus_functions() {
        init();

        // assert something about the repos
        let path_with_subdir = Path::new("/home/sinh/git-repos/others/rust-analyzer/");
        let dir_repos_info = check_dir(&path_with_subdir, &0)
            .await
            .expect("Failed to check test dir");
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

    #[cfg(feature = "dev")]
    #[tokio::test]
    async fn test_check_dir() {
        init();

        // assert something about the repos
        let path_with_subdir = Path::new("/home/sinh/git-repos/andafin/old-projects/");
        let dir_repos_info = check_dir(&path_with_subdir, &0);
        let output_len = dir_repos_info
            .await
            .expect("Failed to check test dir")
            .len();
        assert!(
            output_len == 6,
            "Expected 6 repo, but found {} repos",
            output_len
        );
    }
}
