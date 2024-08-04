use dirs;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct GitRepoInfo {
    pub path: String,
    pub status: String,
    pub origin_url: String,
    pub unpushed_commits: String,
    pub remote_updates: String,
}

impl GitRepoInfo {
    pub fn new(
        path: String,
        origin_url: Option<String>,
        status: String,
        unpushed_commits: String,
        remote_updates: String,
    ) -> Self {
        Self {
            path: path.trim_end_matches('/').to_string(),
            origin_url: origin_url.unwrap_or_default(),
            status,
            unpushed_commits,
            remote_updates,
        }
    }
}

pub struct GitRepoSummary {
    pub path: String,
    pub status_lines: i32,
    pub unpushed_commits_lines: i32,
    pub remote_updates_lines: i32,
}

pub fn save_to_db(repo: &GitRepoInfo) -> Result<()> {
    let db_path = dirs::home_dir()
        .unwrap()
        .join(".local/share/applications/sinh-x/git-status.db");
    println!("db_path: {:?}", &db_path);
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(Path::new(&db_path))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS repos (
                  path TEXT PRIMARY KEY,
                  status TEXT NOT NULL,
                  unpushed_commits TEXT NOT NULL,
                  remote_updates TEXT NOT NULL
                  )",
        params![],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO repos (path, origin_url, status, unpushed_commits, remote_updates) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![repo.path, repo.origin_url, repo.status, repo.unpushed_commits, repo.remote_updates],
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn load_from_db() -> Result<Vec<GitRepoInfo>> {
    let db_path = dirs::home_dir()
        .unwrap()
        .join(".local/share/applications/sinh-x/git-status.db");
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(Path::new(&db_path))?;

    let mut stmt =
        conn.prepare("SELECT path, status, unpushed_commits, remote_updates FROM repos")?;
    let rows = stmt.query_map(params![], |row| {
        Ok(GitRepoInfo {
            path: row.get(0)?,
            status: row.get(1)?,
            unpushed_commits: row.get(2)?,
            remote_updates: row.get(3)?,
        })
    })?;

    let mut repos = Vec::new();
    for repo in rows {
        repos.push(repo?);
    }

    Ok(repos)
}

pub fn summary_repos_table() -> Result<()> {
    let db_path = dirs::home_dir()
        .unwrap()
        .join(".local/share/applications/sinh-x/git-status.db");
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(Path::new(&db_path))?;

    // Create the repos_summary table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS repos_summary (
            path TEXT PRIMARY KEY,
            status_lines INTEGER,
            unpushed_commits_lines INTEGER,
            remote_updates_lines INTEGER
        )",
        [],
    )?;

    // Query the summary line count for each field
    let mut stmt =
        conn.prepare("SELECT path, status, unpushed_commits, remote_updates FROM repos")?;
    let rows = stmt.query_map([], |row| {
        Ok(GitRepoSummary {
            path: row.get::<_, String>(0)?,
            status_lines: (row.get::<_, String>(1)?.matches('\n').count() + 1) as i32,
            unpushed_commits_lines: (row.get::<_, String>(2)?.matches('\n').count() + 1) as i32,
            remote_updates_lines: (row.get::<_, String>(3)?.matches('\n').count() + 1) as i32,
        })
    })?;

    // Insert the results into the repos_summary table
    for row in rows {
        let summary = row?;
        conn.execute(
        "INSERT OR REPLACE INTO repos_summary (path, status_lines, unpushed_commits_lines, remote_updates_lines) VALUES (?, ?, ?, ?)",
        params![summary.path, summary.status_lines, summary.unpushed_commits_lines, summary.remote_updates_lines],
    )?;
    }

    Ok(())
}

pub fn get_summary_stats() -> Result<Vec<GitRepoSummary>> {
    let db_path = dirs::home_dir()
        .unwrap()
        .join(".local/share/applications/sinh-x/git-status.db");
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(Path::new(&db_path))?;
    let mut stmt = conn.prepare("SELECT path, status_lines, unpushed_commits_lines, remote_updates_lines FROM repos_summary")?;

    let rows = stmt.query_map(params![], |row| {
        Ok(GitRepoSummary {
            path: row.get(0)?,
            status_lines: row.get(1)?,
            unpushed_commits_lines: row.get(2)?,
            remote_updates_lines: row.get(3)?,
        })
    })?;

    let mut repos = Vec::new();
    for repo in rows {
        repos.push(repo?);
    }

    Ok(repos)
}
