use bincode;
use dirs;
use log::debug;
use rocksdb::DB;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GitRepoInfo {
    pub path: String,
    pub status: String,
    pub origin_url: String,
    pub unpushed_commits: String,
    pub remote_updates: String,
    pub app_version: Version,
}

impl GitRepoInfo {
    pub fn new(
        path: String,
        origin_url: Option<String>,
        status: String,
        unpushed_commits: String,
        remote_updates: String,
        app_version: Option<Version>,
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
        }
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
            println!("Parsing version: {}", version_str); // Add this line
            println!("Repo: {}", path); // Add this line
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
    db: DB,
    summary_db: DB,
}

impl GitDatabase {
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = std::fs::create_dir_all(path);
        let db = DB::open_default(path.join("repo_db"))?;
        let summary_db = DB::open_default(path.join("summary_db"))?;
        Ok(Self { db, summary_db })
    }

    pub fn save_to_db(&self, repo: &GitRepoInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.db
            .put(repo.path.as_bytes(), bincode::serialize(repo)?)?;
        Ok(())
    }

    pub fn load_from_db(&self) -> Result<Vec<GitRepoInfo>, Box<dyn std::error::Error>> {
        let mut repos = Vec::new();
        for result in self.db.iterator(rocksdb::IteratorMode::Start) {
            let (key, value) = result?;
            let repo: GitRepoInfo = bincode::deserialize(&value)?;
            repos.push(repo);
        }
        Ok(repos)
    }

    pub fn summary_repos_table(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut repos_summary = Vec::new();

        for result in self.db.iterator(rocksdb::IteratorMode::Start) {
            let (key, value) = result?;
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
                .put(summary.path.as_bytes(), bincode::serialize(&summary)?)?;
        }

        Ok(())
    }

    pub fn get_summary_stats(&self) -> Result<Vec<GitRepoSummary>, Box<dyn std::error::Error>> {
        let mut repos = Vec::new();

        for result in self.summary_db.iterator(rocksdb::IteratorMode::Start) {
            let (key, value) = result?;
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
    use std::path::PathBuf;
    use std::sync::Once;

    static INIT: Once = Once::new();
    static TEST_DB_PATH: &str = "/tmp/sinh-x_gitstatus-test.db";

    fn setup() -> GitDatabase {
        INIT.call_once(|| {
            let _ = fs::remove_dir_all(TEST_DB_PATH); // Delete the test database if it exists
        });
        GitDatabase::new(Path::new(TEST_DB_PATH)).unwrap()
    }

    #[test]
    fn test_gitdatabase_version() {
        let repo = GitRepoInfo::new(
            "/path/to/repo".to_string(),
            Some("https://github.com/user/repo.git".to_string()),
            "status".to_string(),
            "unpushed_commits".to_string(),
            "remote_updates".to_string(),
            None,
        );

        let app_version =
            std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION must be set");
        assert!(repo.app_version.to_string().contains(&app_version));
    }

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
        assert_eq!(repos[0], repo, "1::Saved repo does not match loaded repo");

        let repo = GitRepoInfo::new(
            "/path/to/repo".to_string(),
            Some("https://github.com/user/repo.git".to_string()),
            "status-2".to_string(),
            "unpushed_commits-2".to_string(),
            "remote_updates-2".to_string(),
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
        assert_eq!(repos[0], repo, "2::Saved repo does not match loaded repo");

        let repo = GitRepoInfo::new(
            "/path/to/repo-2".to_string(),
            Some("https://github.com/user/repo.git".to_string()),
            "status-2".to_string(),
            "unpushed_commits-2".to_string(),
            "remote_updates-2".to_string(),
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
        assert_eq!(repos[1], repo, "3::Saved repo does not match loaded repo");

        let _ = fs::remove_dir_all(TEST_DB_PATH);
    }
}
