#![feature(exact_size_is_empty)]
use std::{cmp::Ordering, fmt::Display};

use git2::{Diff, StatusOptions};
use log::{debug, warn};
use walkdir::{DirEntry, WalkDir};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");

fn main() {
    // initialize logging early
    env_logger::init();

    // parse command line arguments
    let args = Args::new();
    debug!("{:?}", args);

    let dot = String::from(".");
    let mut count = 0_usize;
    for path in WalkDir::new(".").into_iter().flatten() {
        if is_git(&path) && has_modifications(&args, &path) {
            if let Some(parent) = &path.path().parent() {
                if let Some(name) = parent.to_str() {
                    let real_name = match name {
                        "" => dot.to_owned(),
                        "." => dot.to_owned(),
                        nm => unsafe { nm.to_string().get_unchecked(2..nm.len()).to_string() },
                    };
                    println!("{}", real_name);
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        std::process::exit(100)
    }
}

fn format_dir_entry(path: &DirEntry) -> Option<String> {
    if let Some(parent) = &path.path().parent() {
        if let Some(name) = parent.to_str() {
            let real_name = match name {
                "" => ".".to_string(),
                "." => ".".to_string(),
                nm => unsafe { nm.to_string().get_unchecked(2..nm.len()).to_string() },
            };
            Some(real_name)
        } else {
            None
        }
    } else {
        None
    }
}

fn has_modifications(args: &Args, entry: &DirEntry) -> bool {
    let test = match args.search_kind {
        SearchKind::Index => mods_with_index(entry),
        SearchKind::Status => mods_by_status(entry),
        SearchKind::Tree => mods(entry),
    };
    match test {
        Ok(result) => result,
        Err(e) => {
            let humanized = format_dir_entry(entry).unwrap_or(format!("{:?}", entry));
            warn!("Error in {}: {}", humanized, e);
            false
        }
    }
}

fn mods_with_index(entry: &DirEntry) -> Result<bool, git2::Error> {
    debug!("Scan with index {:?}", entry);
    let repo = git2::Repository::open(entry.path())?;
    let index = repo.index()?;
    let mut opts = git2::DiffOptions::new();
    opts.ignore_submodules(true)
        .include_ignored(false)
        .include_typechange(false);
    let diff = repo.diff_index_to_workdir(Some(&index), Some(&mut opts))?;
    Ok(has_deltas(diff))
}

fn mods_by_status(entry: &DirEntry) -> Result<bool, git2::Error> {
    debug!("Scan with index {:?}", entry);
    let repo = git2::Repository::open(entry.path())?;
    let mut opts = StatusOptions::new();
    opts.include_ignored(false)
        .include_unreadable(false)
        .include_untracked(false)
        .include_ignored(false);
    let stat = repo.statuses(Some(&mut opts))?;
    for st in stat.into_iter() {
        match st.status() {
            git2::Status::CURRENT => {}
            _ => return Ok(true),
        }
    }
    Ok(false)
}

fn mods(entry: &DirEntry) -> Result<bool, git2::Error> {
    debug!("Scan {:?}", entry);
    let repo = git2::Repository::open(entry.path())?;
    let head = repo.head()?;
    if let Some(tgt) = head.target_peel().or(head.target()) {
        let cmt = repo.find_commit(tgt)?;
        let tree = cmt.tree()?;
        let mut opts = git2::DiffOptions::new();
        opts.ignore_submodules(true)
            .include_ignored(false)
            .include_typechange(false);

        let diff = repo.diff_tree_to_workdir(Some(&tree), Some(&mut opts))?;

        return Ok(has_deltas(diff));
    }
    Ok(false)
}

fn has_deltas(diff: Diff) -> bool {
    diff.get_delta(0).is_some()
}

fn is_git(entry: &DirEntry) -> bool {
    if let Some(name) = entry.file_name().to_str() {
        ".git" == name
    } else {
        false
    }
}

#[derive(Debug)]
enum SearchKind {
    Index,
    Status,
    Tree,
}

impl SearchKind {
    fn parse(arg: &str) -> Result<Self, InvalidKind> {
        match arg {
            "i" | "index" => Ok(SearchKind::Index),
            "s" | "status" => Ok(SearchKind::Status),
            "t" | "tree" => Ok(SearchKind::Tree),
            other => Err(InvalidKind {
                name: other.to_string(),
            }),
        }
    }
}

impl Default for SearchKind {
    fn default() -> Self {
        SearchKind::Index
    }
}

#[derive(Debug)]
struct InvalidKind {
    name: String,
}
impl std::error::Error for InvalidKind {}
impl Display for InvalidKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unknown search kind ")?;
        f.write_str(self.name.as_str())
    }
}

#[derive(Debug, Default)]
struct Args {
    search_kind: SearchKind,
}

impl Args {
    fn new() -> Self {
        let mut result = Args::default();
        let args: Vec<String> = std::env::args().collect();

        let h1 = "--help".to_string();
        let h2 = "-h".to_string();
        if args.contains(&h1) || args.contains(&h2) {
            print_help_and_exit(0, "");
        };

        match args.len().cmp(&2_usize) {
            Ordering::Greater => {
                print_help_and_exit(1, "Only one argument should be present");
            }
            Ordering::Equal => match SearchKind::parse(args.get(1).unwrap()) {
                Ok(kind) => result.search_kind = kind,
                Err(e) => print_help_and_exit(2, e),
            },
            _ => {}
        }
        result
    }
}

fn print_help_and_exit(code: i32, message: impl Display) {
    let msg = message.to_string();
    let error = !msg.is_empty();
    if error {
        eprintln!("{}\n", message);
    }

    println(
        error,
        "findmods\n--------\nFind git checkouts containing modifications.\n\n",
    );
    println(error, "Arguments\n---------\n\n");
    println(error, " -h | --help\t\tPrint this help");
    println(
        error,
        " i | index\t\tCompare the index with the working directory (fastest and the default)",
    );
    println(
        error,
        " s | status\t\tGet status for each file and report if any are dirty",
    );
    println(error, " t | tree\t\tCompare the working tree, bypassing the index (detects added but not committed changes)\n");
    println(error, format!("Version:\t{}", VERSION).as_str());
    println(error, format!("Authors:\t{}", AUTHORS).as_str());
    println(error, format!("Origin:\t{}", REPO).as_str());
    println(error, "");

    std::process::exit(code);
}

fn println(err: bool, msg: &str) {
    if err {
        eprintln!("{}", msg)
    } else {
        println!("{}", msg)
    }
}
