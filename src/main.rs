use clap::{arg, Command};

mod irm;
use irm::handle_matches;

fn main() -> std::io::Result<()> {
    let command = Command::new("irm")
        .about("Remove a file or folder from the filesystem")
        .args([
            arg!(-f --force "Force deletion without confirmation"),
            arg!(<FILE> "File or folder to be deleted").required(true),
        ]);

    handle_matches(command.get_matches())?;
    Ok(())
}
