use structopt::StructOpt;
mod git_database;
mod git_status;

use git_database::{get_summary_stats, load_from_db, save_to_db, summary_repos_table, GitRepoInfo};
use git_status::check_dir;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

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
    let args = Cli::from_args();

    match args.command {
        GitCommand::Check { path } => {
            if let Some(repo) = check_dir(&path) {
                println!("Status:\n{}", repo.status);
                println!("Unpushed commits:\n{}", repo.unpushed_commits);
                println!("Updates from remote:\n{}", repo.remote_updates);
                match save_to_db(&repo) {
                    Ok(()) => println!("Saved to database successfully."),
                    Err(e) => eprintln!("Failed to save to database: {}", e),
                }
            }
            summary_repos_table().expect("Failed to create summary table");
        }
        GitCommand::Status => match get_summary_stats() {
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
