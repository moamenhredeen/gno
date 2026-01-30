use std::collections::HashSet;
use std::fs;
use std::path::Path;

use git2::{BranchType, Repository};

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
    // Discover git repository in current directory or parent directories
    let repo = Repository::discover(".")?;

    // Collect statistics
    let total_commits = get_total_commits(&repo)?;
    let branch_count = get_branch_count(&repo)?;
    let contributor_count = get_contributor_count(&repo)?;
    let repo_size = get_repository_size(&repo)?;

    // Display statistics
    display_statistics(total_commits, branch_count, contributor_count, repo_size);

    Ok(())
}

fn get_total_commits(repo: &Repository) -> Result<usize, git2::Error> {
    let mut count = 0;
    let mut visited = HashSet::new();

    // Collect all head OIDs from references
    let mut heads = Vec::new();
    let refs = repo.references()?;

    for reference in refs {
        let reference = reference?;
        if let Some(oid) = reference.target() {
            heads.push(oid);
        }
    }

    // Walk from all heads in a single revwalk
    let mut revwalk = repo.revwalk()?;
    for head in heads {
        revwalk.push(head)?;
    }
    revwalk.set_sorting(git2::Sort::NONE)?;

    for oid in revwalk {
        let oid = oid?;
        if visited.insert(oid) {
            count += 1;
        }
    }

    Ok(count)
}

fn get_branch_count(repo: &Repository) -> Result<usize, git2::Error> {
    let branches = repo.branches(Some(BranchType::Local))?;
    let mut count = 0;

    for branch in branches {
        let _ = branch?;
        count += 1;
    }

    Ok(count)
}

fn get_contributor_count(repo: &Repository) -> Result<usize, git2::Error> {
    let mut contributors = HashSet::new();
    let mut visited = HashSet::new();

    // Collect all head OIDs from references
    let mut heads = Vec::new();
    let refs = repo.references()?;

    for reference in refs {
        let reference = reference?;
        if let Some(oid) = reference.target() {
            heads.push(oid);
        }
    }

    // Walk from all heads in a single revwalk
    let mut revwalk = repo.revwalk()?;
    for head in heads {
        revwalk.push(head)?;
    }
    revwalk.set_sorting(git2::Sort::NONE)?;

    for oid in revwalk {
        let oid = oid?;
        if visited.insert(oid) {
            if let Ok(commit) = repo.find_commit(oid) {
                let author = commit.author();
                let email = author.email().unwrap_or("");
                let name = author.name().unwrap_or("");
                contributors.insert(format!("{} <{}>", name, email));
            }
        }
    }

    Ok(contributors.len())
}

fn get_repository_size(repo: &Repository) -> Result<String, Box<dyn std::error::Error>> {
    let git_dir = repo.path();
    let size = calculate_directory_size(git_dir)?;

    // Format size in human-readable format
    let size_str = if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    };

    Ok(size_str)
}

fn calculate_directory_size(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let mut total_size = 0u64;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                total_size += calculate_directory_size(&entry_path)?;
            } else if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    } else if let Ok(metadata) = fs::metadata(path) {
        total_size += metadata.len();
    }

    Ok(total_size)
}

fn display_statistics(commits: usize, branches: usize, contributors: usize, size: String) {
    println!("Git Repository Statistics");
    println!("{}", "=".repeat(25));
    println!("{:<20} {:>12}", "Total Commits:", format_number(commits));
    println!("{:<20} {:>12}", "Branches:", branches);
    println!("{:<20} {:>12}", "Contributors:", contributors);
    println!("{:<20} {:>12}", "Repository Size:", size);
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    result
}
