#[macro_use]
extern crate clap;
extern crate core;
#[macro_use]
extern crate log;
use clap::Parser;
use gix::{worktree::Index, Repository, ThreadSafeRepository};
use walkdir::{DirEntry, WalkDir};

fn main() {
    // initialize logging early
    env_logger::init();
    let walker = WalkDir::new(".")
        .into_iter()
        // .filter_entry(|e| is_git(e))
        ;

    for path in walker.flatten() {
        if is_git(&path) {
            if has_modifications(&path) {
                println!("{:?}", path);
            }
        }
    }
}

fn has_modifications(entry: &DirEntry) -> bool {
    println!("Scan {:?}", entry.path());
    let p = entry.path().parent().unwrap();
    match git2::Repository::open(entry.path()) {
        Ok(repo) => {
            match repo.index() {
                Ok(index) => {
                    println!("Have index.");
                    
                    
                    
                    let mut opts = git2::DiffOptions::new();
                    opts.ignore_submodules(true)
                        .include_ignored(false)
                        .include_typechange(false);
                    // let trees = repo.worktrees();
                    
                    match repo.diff_tree_to_workdir(None, Some(&mut opts)) {
                        Ok(diff) => {
                            for _i in diff.deltas().into_iter() {
                                return true;
                            }
                            false
                        }
                        Err(e) => {
                            eprintln!("Error diffing tree {:?}: {}", entry.path(), e);
                            false
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error getting index {:?}: {}", entry.path(), e);
                    false
                }
            }
            // }
        }
        Err(e) => {
            eprintln!("Error opening repo {:?}: {}", entry.path(), e);
            false
        }
    }
}

fn x_has_modifications(entry: &DirEntry) -> bool {
    if let Some(dir) = entry.path().parent() {
        match ThreadSafeRepository::open(dir) {
            Ok(repo) => check_modified(repo),
            Err(_) => false,
        }
    } else {
        false
    }
}

fn check_modified(repo: ThreadSafeRepository) -> bool {
    let r = repo.to_thread_local();
    check_index_modified(r)
    // match &r.kind() {
    //     gix::Kind::WorkTree { is_linked } => {
    //         false
    //     },
    //     gix::Kind::Submodule => {
    //         false
    //     },
    //     gix::Kind::Bare => false
    // }
    // if let Some(tree) = repo.worktree() {
    //     match tree.index() {
    //         Ok(index) => check_index_modified(index),
    //         Err(_) => false
    //     }
    // }
}

fn check_index_modified(repo: Repository) -> bool {
    let t = repo.empty_tree();
    // gix_revision::spec::parse(input, delegate)
    match t.changes() {
        Ok(changes) => true,
        Err(_) => false,
    }
}

fn is_git(entry: &DirEntry) -> bool {
    match entry.file_name().to_str() {
        Some(nm) => ".git" == nm,
        None => false,
    }
}
