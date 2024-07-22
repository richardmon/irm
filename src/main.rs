use clap::{arg, Command};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use std::io::stdout;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let command = Command::new("irm")
        .about("Remove a file from the filesystem")
        .arg(arg!(<FILE> "File to be deleted").required(true));

    let matches = command.get_matches();

    if let Some(file) = matches.get_one::<String>("FILE") {
        if std::path::Path::new(file).exists() {
            if std::path::Path::new(file).is_file() {
                println!(
                    "This is a file, and the path is {:?}",
                    std::path::Path::new(file).canonicalize().unwrap()
                );
            } else {
                println!(
                    "This is a directory, and the path is {:?}",
                    std::path::Path::new(file).canonicalize()
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

fn validate_file(file: &PathBuf) -> std::io::Result<u64> {
    let file = file.canonicalize().unwrap();
    Ok(file.metadata().unwrap().len())
}
