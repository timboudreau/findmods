findmods
========

A dirt simple command-line utility to find all git repositories containing modifications.

Sometimes you work on a bunch of related projects, and a common error is to have forgotten local
patches for one project that haven't been committed, and push changes to another project that
depend on those unpushed local changes.

This utility simply list all directories relative to the working directory containing git
repositories with uncommitted changes.  Internally it uses Rust's [`libgit2`](https://libgit2.org/)
wrapper.

I've written versions of this as shell scripts a few times, but this is considerably faster
on large project trees (the number of directories and size of the index are the limiting
factors - for example, BSD's `pkgsrc`, which is huge in folder count and history, is adds
several seconds to a run).

The output is simply a list of folder paths, one per line, with no leading `./`.

The process will exit with `0` if no modifications are found, and `100` if any are.


Arguments
---------

* `-h` | `--help` - print help
* `t` - compare the working tree with the head commit, bypassing the index.  This is 2x slower but
will match checkouts where all changes were added with `git add` but never committed.
* `s` - use the equivalent of `git status` rather than `diff_tree_to_workdir` or `diff_index_to_workdir`;
it is not clear that this option produces and difference other than running slower, and may be removed
in the future.
* `i` - the current default - use the index to compare to the work tree.  This is the fastest performing option,
but will miss added-but-not-committed changes.


Logging
-------

Use `RUST_LOG=debug` for the gory live details of what it's doing; `RUST_LOG=warn` will show if
there are any unreadable/corrupted repositories found.
