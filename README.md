# mksls

[![Build status](https://img.shields.io/github/actions/workflow/status/yanns1/mksls/build_and_test.yml?style=flat-square)](https://github.com/yanns1/mksls/actions/workflows/build_and_test.yml?query=branch%3Amain)
[![Crates.io version](https://img.shields.io/crates/v/mksls?style=flat-square&color=orange)](https://crates.io/crates/mksls)
[![Crates.io downloads](https://img.shields.io/crates/d/mksls?style=flat-square)](https://crates.io/crates/mksls)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue?style=flat-square)](https://docs.rs/mksls)
[![License](https://img.shields.io/badge/license-GPL%203.0-blue?style=flat-square)](https://github.com/yanns1/mksls/blob/main/LICENSE)

mksls is a command-line program that makes the symlinks specified in user-defined files (see [Usecase](#usecase)).
Unix operating systems are supported (e.g. Linux distros, MacOS), but not others (e.g. Windows).

## Contents

- [Usecase](#usecase)
- [Installation](#installation)
- [Usage](#usage)
- [TODO](#todo)

## Usecase

My usecase for this program is in managing my dotfiles.
I put all the configuration files I which to version control in a custom folder (in my case ~/.dotfiles).
I am thus free to organize things like I want instead of having to make Git directories in multiple places, or make a huge directory such as ~/.config tracked by Git with most of the stuff gitignored.

However now, the configuration files are not at the locations they should be, thus the use of symlinks.
All the symlinks could be made by hand, but it is interesting to automate it for at least two reasons:

1. You may wish to version control where the symlinks are made so that you don't have to go over the documentation of each of your tools when you setup your configs for another computer.
1. I might be that one of the symlinks you created long ago gets deleted or overwritten by accident.
   You find it out when you notice your tool not behaving as you thought you configured it.
   You wonder what the cause of the issue is.
   Maybe there is a problem with the symlink!
   Instead of checking the expected location of the configuration file in the tool's documentation, then check whether the proper symlink is present at that location, just run mksls and it is done, whether the symlink was still there, got deleted or overwritten.

You may wonder:

> Why install yet another program for such a simple task that I could have done with a little bit of scripting?

You are right. In fact, my initial inspiration for my current dotfiles setup was [this article](https://shaky.sh/simple-dotfiles/) by Andrew Burgess, which has this [Bash script](https://github.com/andrew8088/dotfiles/blob/main/install/bootstrap.sh) that does almost all of what mksls does (and a bit more at the same time).
If you prefer not installing yet another program and have a script you can easily understand and tweak to your liking, go for it.

## Installation

### Download the executable from the Release page

#### Prerequisites

- None

See <https://github.com/yanns1/mksls/releases>.

### Download mksls from crates.io

#### Prerequisites

- You need Rust installed (more specifically cargo).

Run `cargo install mksls` in your terminal.

### Build from source

#### Prerequisites

- You need Rust installed (more specifically cargo).
- You need Git installed.

Clone this repo and run `./install.sh` from the root of the repo (you might need to give the script executable permission).
`install.sh` builds the project using cargo then make a symlink at ~/.local/bin/mksls targetting the executable produced.

## Usage

Everything is explained in `mksls --help`[^1].

```text
Make symlinks specified in files.

This program makes the symlinks specified in files within DIR having the base FILENAME.
A given file contains zero or more symlink specifications, where a symlink specification is a line with the following format:
    <TARGET_PATH> <SYMLINK_PATH>
Notice the space in between.

There can be multiple spaces, but there needs to be at least one.
If a path contains a space, wrap it in double quotes.
For example, if <TARGET_PATH> contains a space, write this instead:
     "<TARGET_PATH>" <SYMLINK_PATH>
If you have a double quote in one of the paths... Change it!

By default, the program is interactive.
If no file is found where a given symlink is about to be made, the symlink will be made.
However, if a file is found, you will be asked to choose between:
    [s]kip : Don't create the symlink and move on to the next one.
    [S]kip all : [s]kip for the current symlink and all further symlink conflicting with an existing file.
    [b]ackup : Move the existing file in BACKUP_DIR, then make the current symlink.
    [B]ackup all : [b]ackup for the current symlink and all further symlink conflicting with an existing file.
    [o]verwrite : Overwrite the existing file with the symlink (beware data loss!)
    [O]verwrite all : [o]verwrite for the current symlink and all further symlink conflicting with an existing file.
However it can be made uninteractive by using one (and only one) of these options:
    --always-skip (equivalent to always selecting 's')
    --always-backup (equivalent to always selecting 'b')
There is no --always-overwrite for you to not regret it.

For each processed symlink specification, a line with the following format is printed:
    (<action>) <link> -> <target>
where <action> encodes what has been done for that symlink:
    . : Already existed, so has been skipped.
    d : Done. The symlink was successfully created.
    s : There was a conflict between the link and an existing file, and choose to [s]kip.
    b : There was a conflict between the link and an existing file, and choose to [b]ackup.
    o : There was a conflict between the link and an existing file, and choose to [o]verwrite.
(<link> and <target> are respectively the link and target of the symlink specification)

Usage: mksls [OPTIONS] <DIR>

Arguments:
  <DIR>
          The directory in which to scan for files specifying symlinks.

Options:
  -f, --filename <FILENAME>
          The base (name + extension) of the file(s) specifying symlinks to make.

          By default, the name is "sls".
          If one is specified in the config file, it will be used instead.

  -b, --backup-dir <BACKUP_DIR>
          The backup directory in which to store the backed up files during execution.

          By default, it is set to:
              (Linux) $XDG_CONFIG_HOME/mksls/backups/ or .config/mksls/backups/ if $XDG_CONFIG_HOME is not set
              (Mac) $HOME/Library/Application Support/mksls/backups/

      --always-skip
          Always skip the symlinks conflicting with an existing file.

          This makes the program uninteractive.
          Of course, it can't be combined with --always-backup.

      --always-backup
          Always backup the conflicting file before replacing it by the symlink.

          This makes the program uninteractive.
          Of course, it can't be combined with --always-skip.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Configuration file:
You can provide other default values for the options:
    --filename
    --backup-dir
    --always-skip
    --always-backup
in a TOML configuration file located at:
    (Linux) $XDG_CONFIG_HOME/<project_path> or .config/<project_path> if $XDG_CONFIG_HOME is not set
    (Mac) $HOME/Library/Application Support/<project_path>
where <project_path> is 'mksls/mksls.toml'.

Note:
    - If you didn't write a config file yourself, one with the default values will automatically be written.
    - Paths in the config file should be absolute.
```

## TODO

- *Make shell completions*: Seems to cumbersome.

- *Integration tests*: Cumbersome given that many uses of the app are interactive.
  Furthermore, the codebase is relatively small, so I am rather confident in it.
  But in case I want to do integration tests, here are useful ideas:

  - Before starting to write tests, think about what is worth testing, partition
    the input space wisely.
  - Making the application's code more generic can help for testing (for example
    being generic over `Read` and `Write` traits).
    See <https://rust-cli.github.io/book/tutorial/testing.html>.
  - [assert_fs](https://docs.rs/assert_fs/latest/assert_fs/): Filesystem fixtures
    and assertions for testing.
  - [assert_cmd](https://docs.rs/assert_cmd/latest/assert_cmd/): Easy command
    initialization and assertions.
  - [predicates](https://docs.rs/predicates/latest/predicates/): Composable
    first-order predicate functions.
  - [lit](https://docs.rs/lit/latest/lit/): A reusable testing tool, inspired by
    LLVM’s lit tool. It defines a DSL for matching text outputs easily.
    See <https://www.neilhenning.dev/posts/rust-lit/> also.
  - [rexpect](https://docs.rs/rexpect/latest/rexpect/): Assert against the output
    of an interactive CLI, and send input back to it. See
    <https://www.rustadventure.dev/building-a-digital-garden-cli/clap-v4/testing-interactive-clis-with-rexpect>
    for a simple tutorial.

[^1]: In fact the full help is so detailed that you will certainly prefer the shorter version from `mksls -h` once you get the gist of it.
