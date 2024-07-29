use clap::ArgMatches;
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use std::io::stdout;
use std::path::PathBuf;

pub fn handle_matches(matches: ArgMatches) -> std::io::Result<()> {
    let is_force: bool = matches.get_flag("force");
    if let Some(file) = matches.get_one::<String>("FILE") {
        if std::path::Path::new(file).exists() {
            if std::path::Path::new(file).is_file() {
                println!(
                    "This is a file, and the path is {:?}, is it force delete? {}",
                    std::path::Path::new(file).canonicalize().unwrap(),
                    is_force
                );
                let file_mode = std::fs::metadata(file)?.permissions();
                if file_mode.readonly() {
                    if is_force {
                        std::fs::remove_file(file)?;
                    } else {
                        execute!(
                            stdout(),
                            SetForegroundColor(Color::Red),
                            Print(format!(
                                "The file {} is read-only, use the force flag -f to delete it",
                                file
                            )),
                            ResetColor
                        )?;
                        return Ok(());
                    }
                } else {
                    std::fs::remove_file(file)?;
                }
                execute!(
                    stdout(),
                    SetForegroundColor(Color::Green),
                    Print(format!("The file {} has been deleted!\n", file)),
                    ResetColor
                )?;
            } else {
                println!(
                    "This is a directory, and the path is {:?}, is it force delete? {}",
                    std::path::Path::new(file).canonicalize(),
                    is_force
                );
                let size = validate_directory(&std::path::PathBuf::from(file))?;
                execute!(
                    stdout(),
                    SetForegroundColor(Color::Green),
                    Print(format!(
                        "The size of the directory is: {}\n",
                        human_readable_size(size)
                    )),
                    ResetColor
                )?;

                let items_to_delte = walkdir::WalkDir::new(file).into_iter();
                // Start the progress bar
                let pb = indicatif::ProgressBar::new(size);
                pb.set_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) \n{msg}").unwrap()
                        .progress_chars("#>-")
                    );
                pb.set_position(0);

                let mut progress = 0;
                for entry in items_to_delte {
                    let entry = entry.unwrap();
                    if entry.path().is_file() {
                        let metadata = entry.metadata().unwrap();
                        let mode = metadata.permissions();
                        if mode.readonly() {
                            execute!(
                                stdout(),
                                SetForegroundColor(Color::Red),
                                Print(format!(
                                    "The file {:#} is read-only, use the force flag to delete it",
                                    entry.path().display()
                                )),
                                ResetColor,
                            )?;
                            if is_force {
                                std::fs::remove_file(entry.path()).unwrap();
                            } else {
                                pb.finish_with_message(format!(
                                    "The file {:#} is read-only, use the force flag to delete it",
                                    entry.path().display()
                                ));
                                return Ok(());
                            }
                        } else {
                            std::fs::remove_file(entry.path()).unwrap();
                        }
                        progress += metadata.len();
                        pb.set_position(progress);
                        pb.set_message(format!("Deleting: {}", entry.path().display()));
                        // slip for a second
                        // std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
                std::fs::remove_dir(&file)?;
                pb.set_message("");
                pb.finish();
            }
        } else {
            execute!(
                stdout(),
                SetForegroundColor(Color::Red),
                Print(format!("The file {:#} does not exist!. \n", file)),
                ResetColor
            )?;
        }
    }
    Ok(())
}

fn validate_directory(folder: &PathBuf) -> std::io::Result<u64> {
    let folder = folder.canonicalize().unwrap();
    let mut size = 0;

    for entry in walkdir::WalkDir::new(folder) {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            size += entry.metadata().unwrap().len();
        }
    }
    Ok(size)
}

/**
* return a text representation of the size of the a file or directory
*/
fn human_readable_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut size = size as f64;
    let mut unit = 0;
    while size >= 1024.0 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", size, units[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_directory_size_zero() {
        // Create a temporary directory
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let temp_file = temp_dir_path.join("temp_file.txt");
        std::fs::File::create(&temp_file).unwrap();
        let temp_sub_dir = temp_dir_path.join("temp_sub_dir");
        std::fs::create_dir(&temp_sub_dir).unwrap();
        let temp_sub_file = temp_sub_dir.join("temp_sub_file.txt");
        std::fs::File::create(&temp_sub_file).unwrap();

        assert_eq!(validate_directory(&temp_dir_path).unwrap(), 0);
    }

    #[test]
    fn test_validate_directory_size_non_zero() {
        // Create a temporary directory
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let temp_file = temp_dir_path.join("temp_file.txt");
        std::fs::File::create(&temp_file).unwrap();
        let temp_sub_dir = temp_dir_path.join("temp_sub_dir");
        std::fs::create_dir(&temp_sub_dir).unwrap();
        let temp_sub_file = temp_sub_dir.join("temp_sub_file.txt");
        std::fs::File::create(&temp_sub_file).unwrap();

        // fill the file with some data
        let data = "Hello, World!";
        std::fs::write(&temp_sub_file, data).unwrap();
        let expected_size = data.len() as u64;

        assert_eq!(validate_directory(&temp_dir_path).unwrap(), expected_size);
    }

    #[test]
    fn test_human_readable_b() {
        assert_eq!(human_readable_size(1), "1.00 B");
    }

    #[test]
    fn test_human_readable_kb() {
        assert_eq!(human_readable_size(1024), "1.00 KB");
    }

    #[test]
    fn test_human_readable_mb() {
        assert_eq!(human_readable_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_human_readable_gb() {
        assert_eq!(human_readable_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_human_readable_tb() {
        assert_eq!(human_readable_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
    }
}
