use clap::Parser;

#[derive(Parser)]
#[command(name = "mksls")]
#[command(version, about, long_about = None)]
struct Cli {
    /// The directory in which to scan for files specifying symlinks.
    #[clap(verbatim_doc_comment)]
    dir: String,

    /// The name (including the extension) of the file(s) specifying symlinks to make.
    ///
    /// By default, the name is "sls".
    /// If one is specified in the config file, it will be used instead.
    ///
    /// A symlink is specified on a single line, with the following syntax:
    ///     <TARGET_PATH> <SYMLINK_PATH>
    /// Notice the space separating them.
    /// There can be multiple spaces, but there needs to be at least one.
    /// If a path contains a space, wrap it in double quotes.
    /// For example, if <TARGET_PATH> contains a space, write this instead:
    ///     "<TARGET_PATH>" <SYMLINK_PATH>
    #[clap(verbatim_doc_comment)]
    #[arg(short, long)]
    filename: Option<String>,

    /// The depth up to which files specifying symlinks to make will be considered.
    ///     
    /// By default, depth is unlimited, meaning the program will search as deep as
    /// it can in the input directory.
    #[clap(verbatim_doc_comment)]
    #[arg(short, long)]
    depth: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    println!("filename: {:?}", cli.filename);
}
