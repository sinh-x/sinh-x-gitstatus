mod config;
mod git_database;
mod git_status;

use config::Config;
use env_logger::Env;
use git_database::GitDatabase;
use git_status::check_dir;
use log::debug;
use std::fs;
use std::io::Write;
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
    },
    #[structopt(about = "Load the status of all git repositories from the database.")]
    Status,
}

fn main() {
    let config_path = Path::new("path/to/config.toml");
    let config = if config_path.exists() {
        // Load the config from the file
        Config::new(config_path).unwrap()
    } else {
        // Create a new config with default values
        let config = Config::new(config_path).expect("Failed to load config");
        config.validate().expect("Invalid config");

        config
    };

    let binding = config.general.database_path.unwrap();
    let db_path = Path::new(&binding);
    let gitdb = GitDatabase::new(db_path).unwrap();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Cli::from_args();

    match args.command {
        GitCommand::Check { path } => {
            let repos = check_dir(&path);
            for repo in repos {
                debug!("Status:\n{}", repo.status);
                debug!("Unpushed commits:\n{}", repo.unpushed_commits);
                debug!("Updates from remote:\n{}", repo.remote_updates);
                match gitdb.save_to_db(&repo) {
                    Ok(()) => println!("Saved to database successfully."),
                    Err(e) => eprintln!("Failed to save to database: {}", e),
                }
            }
            gitdb
                .summary_repos_table()
                .expect("Failed to create summary table");
        }
        GitCommand::Status => match gitdb.get_summary_stats() {
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
            Err(e) => eprintln!("Failed to load from database: {}", e),
        },
    }
}
