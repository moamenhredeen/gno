mod stats;

use clap::{Arg, Command, crate_authors, crate_description, crate_version};
use git2::Repository;

use crate::stats::{
    format_number, get_branch_count, get_contributor_count, get_repository_size, get_total_commits,
};

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("gno")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand_required(true)
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Config file path")
                .long_help("Path to the config file")
                .default_value("gno.toml"),
        )
        .arg(
            Arg::new("path")
                .long("path")
                .short('p')
                .help("Path to the git repository")
                .long_help("Path to the git repository")
                .default_value("."),
        )
        .subcommand(
            Command::new("commits")
                .alias("c")
                .about("commit statistics"),
        )
        .subcommand(
            Command::new("contributors")
                .alias("cont")
                .about("show contributor statistics"),
        )
        .subcommand(
            Command::new("summary")
                .alias("s")
                .about("show summary of the repository statistics"),
        )
        .get_matches();

    // Discover git repository in current directory or parent directories
    let repo = Repository::discover(".")?;

    match cmd.subcommand() {
        Some(("commits", _)) => {
            let total_commits = get_total_commits(&repo)?;
            println!("Git Repository Statistics");
            println!("{}", "=".repeat(25));
            println!(
                "{:<20} {:>12}",
                "Total Commits:",
                format_number(total_commits)
            );
        }
        Some(("branches", _)) => {
            let branch_count = get_branch_count(&repo)?;
            println!("Git Repository Statistics");
            println!("{}", "=".repeat(25));
            println!("{:<20} {:>12}", "Branches:", branch_count);
        }
        Some(("contributors", _)) => {
            let contributor_count = get_contributor_count(&repo)?;
            println!("Git Repository Statistics");
            println!("{}", "=".repeat(25));
            println!("{:<20} {:>12}", "Contributors:", contributor_count);
        }
        Some(("summary", _)) => {
            let repo_size = get_repository_size(&repo)?;
            println!("Git Repository Statistics");
            println!("{}", "=".repeat(25));
            println!("{:<20} {:>12}", "Repository Size:", repo_size);
        }
        _ => {}
    };
    Ok(())
}
