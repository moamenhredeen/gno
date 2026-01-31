use std::{collections::HashSet, fs, path::Path};

use git2::{BranchType, Repository};

pub fn get_total_commits(repo: &Repository) -> Result<usize, git2::Error> {
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

pub fn get_branch_count(repo: &Repository) -> Result<usize, git2::Error> {
    let branches = repo.branches(Some(BranchType::Local))?;
    let mut count = 0;

    for branch in branches {
        let _ = branch?;
        count += 1;
    }

    Ok(count)
}

pub fn get_contributor_count(repo: &Repository) -> Result<usize, git2::Error> {
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

pub fn get_repository_size(repo: &Repository) -> Result<String, Box<dyn std::error::Error>> {
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

pub fn calculate_directory_size(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
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

pub fn format_number(n: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Signature;
    use tempfile::TempDir;

    #[test]
    fn test_repository_with_no_commits() {
        let td = TempDir::new().unwrap();
        let path = td.path();
        let repo = Repository::init(path).unwrap();
        let commit_count = get_total_commits(&repo).unwrap();
        assert_eq!(0, commit_count);
    }

    #[test]
    fn test_repository_with_one_commit() {
        let td = TempDir::new().unwrap();
        let path = td.path();

        let repo = Repository::init(path).unwrap();
        let sig = Signature::now("test", "test@example.com").unwrap();

        let mut index = repo.index().unwrap();
        let p = Path::new(repo.workdir().unwrap()).join("file_a");
        fs::File::create(&p).unwrap();
        index.add_path(Path::new("file_a")).unwrap();
        let id_a = index.write_tree().unwrap();
        let tree_a = repo.find_tree(id_a).unwrap();
        repo.commit(
            Some("refs/heads/branch_a"),
            &sig,
            &sig,
            "commit 2",
            &tree_a,
            &[],
        )
        .unwrap();

        let commit_count = get_total_commits(&repo).unwrap();
        assert_eq!(1, commit_count);
    }

    #[test]
    fn test_repository_with_more_than_one_commit() {
        let td = TempDir::new().unwrap();
        let path = td.path();

        let repo = Repository::init(path).unwrap();
        let sig = Signature::now("test", "test@example.com").unwrap();

        let mut index = repo.index().unwrap();
        let p = Path::new(repo.workdir().unwrap()).join("file_a");
        fs::File::create(&p).unwrap();
        index.add_path(Path::new("file_a")).unwrap();
        let id_a = index.write_tree().unwrap();
        let tree_a = repo.find_tree(id_a).unwrap();
        let oid1 = repo
            .commit(
                Some("refs/heads/branch_a"),
                &sig,
                &sig,
                "commit 1",
                &tree_a,
                &[],
            )
            .unwrap();
        let commit1 = repo.find_commit(oid1).unwrap();

        let p = Path::new(repo.workdir().unwrap()).join("file_b");
        fs::File::create(&p).unwrap();
        index.add_path(Path::new("file_b")).unwrap();
        let id_b = index.write_tree().unwrap();
        let tree_b = repo.find_tree(id_b).unwrap();
        repo.commit(
            Some("refs/heads/branch_a"),
            &sig,
            &sig,
            "commit 2",
            &tree_b,
            &[&commit1],
        )
        .unwrap();

        let commit_count = get_total_commits(&repo).unwrap();
        assert_eq!(2, commit_count);
    }
}
