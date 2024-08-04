use structopt::StructOpt;
mod git_database;
mod git_status;

use env_logger::Env;
use git_database::{get_summary_stats, save_to_db, summary_repos_table};
use git_status::check_dir;
use log::debug;
use std::path::PathBuf;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Cli::from_args();

    match args.command {
        GitCommand::Check { path } => {
            let repos = check_dir(&path);
            for repo in repos {
                debug!("Status:\n{}", repo.status);
                debug!("Unpushed commits:\n{}", repo.unpushed_commits);
                debug!("Updates from remote:\n{}", repo.remote_updates);
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
