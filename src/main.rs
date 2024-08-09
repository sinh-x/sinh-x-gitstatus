mod config;
mod git_database;
mod git_status;

use config::Config;
use env_logger::Env;
use git2::Repository;
use git_database::GitDatabase;
use git_status::check_dir;
use log::debug;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[macro_use]
extern crate serde_derive;

#[derive(Debug, StructOpt)]
#[structopt(name = "gitstatus", about = "Checks the status of git repositories.")]
struct Cli {
    #[structopt(subcommand)]
    command: GitCommand,
}

#[derive(Debug, StructOpt)]
enum GitCommand {
    #[structopt(about = "Check the status of a git repository and save it to the database.")]
    Check {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        #[structopt(short = "L", long, default_value = "0", validator = validate_detail_level)]
        detail_level: u8,
    },
    #[structopt(about = "Load the status of all git repositories from the database.")]
    Status {
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
    },
    Commits,
}

fn validate_detail_level(level: String) -> Result<(), String> {
    match level.parse::<u8>() {
        Ok(val) if val <= 1 => Ok(()),
        _ => Err(String::from(
            "Detail level must be a number between 0 and 1",
        )),
    }
}

fn get_absolute_path(path: &Path) -> std::io::Result<PathBuf> {
    let absolute_path = fs::canonicalize(path)?;
    Ok(absolute_path)
}

#[tokio::main]
async fn main() {
    let config_path: PathBuf = dirs::home_dir().unwrap();
    debug!("Home dir: {:?}", config_path);
    let config_path = config_path.join(".config/sinh-x/gitstatus/config.toml");
    let config = if config_path.exists() {
        // Load the config from the file
        let config = Config::new(config_path.as_path()).unwrap();
        config.validate().expect("Invalid config");
        config
    } else {
        // Create a new config with default values
        Config::default()
    };

    let binding = config.general.database_path.unwrap();
    let db_path = Path::new(&binding);
    let gitdb = GitDatabase::new(db_path).unwrap();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Cli::from_args();

    match args.command {
        GitCommand::Check { path, detail_level } => {
            let absolute_path = get_absolute_path(path.as_path());
            match check_dir(&absolute_path.unwrap(), &detail_level, &gitdb).await {
                Ok(repos) => {
                    for repo in repos {
                        debug!("Status:\n{}", repo.status);
                        debug!("Unpushed commits:\n{}", repo.unpushed_commits);
                        debug!("Updates from remote:\n{}", repo.remote_updates);
                        match gitdb.save_to_db(&repo) {
                            Ok(()) => debug!("Saved to database successfully: {}", repo.path),
                            Err(e) => eprintln!("Failed to save to database: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("Check Command Failed: {}", e),
            }
        }
        GitCommand::Status { path } => {
            match path {
                Some(path) => {
                    // Handle the case where path is Some
                    let absolute_path = get_absolute_path(path.as_path());
                    // For example, you might want to print the path:
                    match gitdb.get_repo_details(absolute_path.expect("Path failed")) {
                        Ok(repo_info) => {
                            println!("Test {}", repo_info.path);
                            println!("Test {}", repo_info.app_version);
                            for commit in &repo_info.commits.unwrap_or_default() {
                                println!("Commit: {}", commit);
                                println!(
                                    "{} | {} | {} ",
                                    commit.file_changes, commit.insertions, commit.deletion,
                                );
                            }
                        }
                        Err(e) => eprintln!("Status Command 1 Repo Failed: {}", e),
                    }
                }
                None => match gitdb.get_summary_stats() {
                    Ok(repos) => {
                        for repo in repos {
                            println!(
                                "{} | {} | {} | {}",
                                repo.path,
                                repo.status_lines,
                                repo.unpushed_commits_lines,
                                repo.remote_updates_lines,
                            );
                        }
                    }
                    Err(e) => eprintln!("Status Commnd - All Failed: {}", e),
                },
            }
            let _ = gitdb.summary_repos_table();
        }
        GitCommand::Commits => print_all_commits(".").expect("Failed to print commits"),
    }
}

fn print_all_commits(repo_path: &str) -> Result<(), git2::Error> {
    let repo = Repository::open(repo_path)?;

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

        println!(
            "{} - {}: {} ({} - {} - {})",
            id,
            commit.author(),
            commit.message().unwrap_or("No commit message"),
            stats.files_changed(),
            stats.insertions(),
            stats.deletions()
        );
    }

    Ok(())
}
