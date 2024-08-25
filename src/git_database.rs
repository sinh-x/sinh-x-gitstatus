use bincode;
use colored::*;
use git2::Time;
use log::debug;
use semver::Version;
use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use sled::Db;
use std::fmt;
use std::path::{Path, PathBuf};
use tokei::Languages;

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepoInfo {
    pub path: String,
    pub status: String,
    pub origin_url: String,
    pub unpushed_commits: String,
    pub remote_updates: String,
    pub app_version: Version,
    pub commits: Option<Vec<GitCommit>>,
    pub languages: Option<Languages>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepoInfoV051 {
    pub path: String,
    pub status: String,
    pub origin_url: String,
    pub unpushed_commits: String,
    pub remote_updates: String,
    pub app_version: Version,
    pub commits: Option<Vec<GitCommit>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepoInfoV030 {
    pub path: String,
    pub status: String,
    pub origin_url: String,
    pub unpushed_commits: String,
    pub remote_updates: String,
    pub app_version: Version,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub author_email: String,
    pub time: SerializableTime,
    pub message: String,
    pub file_changes: usize,
    pub insertions: usize,
    pub deletion: usize,
}

#[derive(Debug, PartialEq)]
pub struct SerializableTime(pub Time);

#[derive(Debug)]
pub enum GitDatabaseError {
    KeyNotExist,
    SledError(sled::Error),
    BinCodeError(bincode::Error),
}

impl std::fmt::Display for GitDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GitDatabaseError::KeyNotExist => write!(f, "Key not existed in database."),
            GitDatabaseError::SledError(err) => write!(f, "sled error: {}", err),
            GitDatabaseError::BinCodeError(err) => write!(f, "bincode error: {}", err),
        }
    }
}

impl From<sled::Error> for GitDatabaseError {
    fn from(err: sled::Error) -> GitDatabaseError {
        GitDatabaseError::SledError(err)
    }
}
impl From<bincode::Error> for GitDatabaseError {
    fn from(err: bincode::Error) -> GitDatabaseError {
        GitDatabaseError::BinCodeError(err)
    }
}

impl serde::Serialize for SerializableTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.0.seconds())
    }
}

impl<'de> serde::Deserialize<'de> for SerializableTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = i64::deserialize(deserializer)?;
        Ok(SerializableTime(Time::new(seconds, 0)))
    }
}

impl GitCommit {
    pub fn new(
        hash: String,
        author_email: String,
        time: SerializableTime,
        message: String,
        file_changes: usize,
        insertions: usize,
        deletion: usize,
    ) -> Self {
        Self {
            hash,
            author_email,
            time,
            message,
            file_changes,
            insertions,
            deletion,
        }
    }
}

impl fmt::Display for GitCommit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Hash: {}, Author Email: {}, Message: {}",
            self.hash, self.author_email, self.message
        )
    }
}
impl GitRepoInfo {
    pub fn new(
        path: String,
        origin_url: Option<String>,
        status: String,
        unpushed_commits: String,
        remote_updates: String,
        app_version: Option<Version>,
        commits: Option<Vec<GitCommit>>,
        languages: Option<Languages>,
    ) -> Self {
        let app_version = app_version.unwrap_or_else(|| {
            let version_str = env!("CARGO_PKG_VERSION");
            debug!("Parsing version: {}", version_str); // Add this line
            debug!("Repo: {}", path); // Add this line
            Version::parse(version_str).unwrap()
        });
        Self {
            path: path.trim_end_matches('/').to_string(),
            origin_url: origin_url.unwrap_or_default(),
            status,
            unpushed_commits,
            remote_updates,
            app_version,
            commits,
            languages,
        }
    }
    //TODO: implemment is_equal for all field.
    fn is_equal(&self, other: &Self) -> bool {
        self.path == other.path
            && self.origin_url == other.origin_url
            && self.status == other.status
            && self.unpushed_commits == other.unpushed_commits
            && self.remote_updates == other.remote_updates
            && self.app_version == other.app_version
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepoSummary {
    pub path: String,
    pub origin_url: String,
    pub status_lines: i32,
    pub unpushed_commits_lines: i32,
    pub remote_updates_lines: i32,
    pub app_version: Version,
}

impl GitRepoSummary {
    pub fn new(
        path: String,
        origin_url: Option<String>,
        status_lines: i32,
        unpushed_commits_lines: i32,
        remote_updates_lines: i32,
    ) -> Self {
        let app_version = {
            let version_str = env!("CARGO_PKG_VERSION");
            debug!("Parsing version: {}", version_str); // Add this line
            debug!("Repo: {}", path); // Add this line
            Version::parse(version_str).unwrap()
        };
        Self {
            path: path.trim_end_matches('/').to_string(),
            origin_url: origin_url.unwrap_or_default(),
            status_lines,
            unpushed_commits_lines,
            remote_updates_lines,
            app_version,
        }
    }
}
pub struct GitDatabase {
    db: Db,
    summary_db: Db,
}

impl Clone for GitDatabase {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            summary_db: self.summary_db.clone(),
        }
    }
}

impl GitDatabase {
    pub fn new(path: &Path) -> Result<Self, GitDatabaseError> {
        let _ = std::fs::create_dir_all(path);
        let db = sled::open(path.join("repo_db"))?;
        let summary_db = sled::open(path.join("summary_db"))?;
        Ok(Self { db, summary_db })
    }

    pub fn save_to_db(&self, repo: &GitRepoInfo) -> Result<(), GitDatabaseError> {
        self.db
            .insert(repo.path.as_bytes(), bincode::serialize(repo)?)?;
        Ok(())
    }

    //TODO: Review this function
    #[allow(dead_code)]
    pub fn load_from_db(&self) -> Result<Vec<GitRepoInfo>, GitDatabaseError> {
        let mut repos = Vec::new();
        for result in self.db.iter() {
            let (_key, value) = result?;
            let repo: GitRepoInfo = Self::deserialize_git_repo_info(&value)?;
            repos.push(repo);
        }
        Ok(repos)
    }

    fn deserialize_git_repo_info(data: &[u8]) -> Result<GitRepoInfo, bincode::Error> {
        match bincode::deserialize::<GitRepoInfo>(data) {
            Ok(repo) => Ok(repo),
            Err(e) => match *e {
                bincode::ErrorKind::Io(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    match bincode::deserialize::<GitRepoInfoV051>(data) {
                        Ok(repo_v051) => {
                            debug!(
                            "{}",
                            format!("WARNING: Old version of data {}. Please run with Check command to update the repo!", repo_v051.app_version).yellow()
                        );
                            let new_repo = GitRepoInfo::new(
                                repo_v051.path,
                                Some(repo_v051.origin_url),
                                repo_v051.status,
                                repo_v051.unpushed_commits,
                                repo_v051.remote_updates,
                                Some(repo_v051.app_version),
                                repo_v051.commits,
                                None,
                            );
                            Ok(new_repo)
                        }
                        Err(e) => match *e {
                            bincode::ErrorKind::Io(ref e)
                                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                            {
                                let repo_v030: GitRepoInfoV030 = bincode::deserialize(data)?;
                                debug!(
                                "{}",
                                format!("WARNING: Old version of data {}. Please run with Check command to update the repo!", repo_v030.app_version).yellow()
                            );
                                let new_repo = GitRepoInfo::new(
                                    repo_v030.path,
                                    Some(repo_v030.origin_url),
                                    repo_v030.status,
                                    repo_v030.unpushed_commits,
                                    repo_v030.remote_updates,
                                    Some(repo_v030.app_version),
                                    None,
                                    None,
                                );
                                Ok(new_repo)
                            }
                            _ => Err(e),
                        },
                    }
                }
                _ => Err(e),
            },
        }
    }

    pub fn get_repo_details(&self, path: PathBuf) -> Result<GitRepoInfo, GitDatabaseError> {
        match self.db.get(path.display().to_string()) {
            Ok(Some(value)) => match Self::deserialize_git_repo_info(&value) {
                Ok(repo) => Ok(repo),
                Err(e) => Err(GitDatabaseError::BinCodeError(e)),
            },
            Ok(None) => Err(GitDatabaseError::KeyNotExist),
            Err(e) => {
                debug!("git_repo_details: data handling error!");
                Err(GitDatabaseError::SledError(e))
            }
        }
    }

    pub fn summary_repos_table(&self) -> Result<(), GitDatabaseError> {
        let mut repos_summary = Vec::new();

        for result in self.db.iter() {
            let (_key, value) = result?;
            let repo: GitRepoInfo = bincode::deserialize(&value)?;
            debug!("{:?}", repo.path);
            let summary = GitRepoSummary::new(
                repo.path,
                Some(repo.origin_url.clone()),
                repo.status.matches('\n').count() as i32,
                repo.unpushed_commits.matches('\n').count() as i32,
                repo.remote_updates.matches('\n').count() as i32,
            );
            repos_summary.push(summary);
        }

        for summary in repos_summary {
            self.summary_db
                .insert(summary.path.as_bytes(), bincode::serialize(&summary)?)?;
        }

        Ok(())
    }

    pub fn get_summary_stats(&self) -> Result<Vec<GitRepoSummary>, Box<dyn std::error::Error>> {
        let mut repos = Vec::new();

        for result in self.summary_db.iter() {
            let (_key, value) = result?;
            let summary: GitRepoSummary = bincode::deserialize(&value)?;
            repos.push(summary);
        }

        Ok(repos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Once;

    static INIT: Once = Once::new();
    static TEST_DB_PATH: &str = "/tmp/sinh-x_gitstatus-test.db";

    fn setup() -> GitDatabase {
        INIT.call_once(|| {
            let _ = fs::remove_dir_all(TEST_DB_PATH); // Delete the test database if it exists
        });
        GitDatabase::new(Path::new(TEST_DB_PATH)).unwrap()
    }

    #[cfg(feature = "dev")]
    #[test]
    fn test_gitdatabase_version() {
        let repo = GitRepoInfo::new(
            "/path/to/repo".to_string(),
            Some("https://github.com/user/repo.git".to_string()),
            "status".to_string(),
            "unpushed_commits".to_string(),
            "remote_updates".to_string(),
            None,
            None,
            None,
        );

        let app_version =
            std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION must be set");
        assert!(repo.app_version.to_string().contains(&app_version));
    }

    #[cfg(feature = "dev")]
    #[test]
    fn test_gitdatabase() {
        let db = setup();

        let repo = GitRepoInfo::new(
            "/path/to/repo".to_string(),
            Some("https://github.com/user/repo.git".to_string()),
            "status".to_string(),
            "unpushed_commits".to_string(),
            "remote_updates".to_string(),
            None,
            None,
            None,
        );

        db.save_to_db(&repo).unwrap();

        // Verify that the repo was saved correctly
        let repos = db.load_from_db().unwrap();
        assert_eq!(
            repos.len(),
            1,
            "1::Expected 1 repo, but found {}",
            repos.len()
        );
        assert!(
            repos[0].is_equal(&repo),
            "1::Saved repo does not match loaded repo"
        );

        let repo = GitRepoInfo::new(
            "/path/to/repo".to_string(),
            Some("https://github.com/user/repo.git".to_string()),
            "status-2".to_string(),
            "unpushed_commits-2".to_string(),
            "remote_updates-2".to_string(),
            None,
            None,
            None,
        );

        db.save_to_db(&repo).unwrap();

        // Verify that the repo was saved correctly
        let repos = db.load_from_db().unwrap();
        assert_eq!(
            repos.len(),
            1,
            "2::Expected 1 repo, but found {}",
            repos.len()
        );
        assert!(
            repos[0].is_equal(&repo),
            "2::Saved repo does not match loaded repo"
        );

        let repo = GitRepoInfo::new(
            "/path/to/repo-2".to_string(),
            Some("https://github.com/user/repo.git".to_string()),
            "status-2".to_string(),
            "unpushed_commits-2".to_string(),
            "remote_updates-2".to_string(),
            None,
            None,
            None,
        );

        db.save_to_db(&repo).unwrap();

        // Verify that the repo was saved correctly
        let repos = db.load_from_db().unwrap();
        assert_eq!(
            repos.len(),
            2,
            "3::Expected 2 repo, but found {}",
            repos.len()
        );
        assert!(
            repos[1].is_equal(&repo),
            "3::Saved repo does not match loaded repo"
        );

        let _ = fs::remove_dir_all(TEST_DB_PATH);
    }
}
