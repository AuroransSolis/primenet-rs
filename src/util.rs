use std::error::Error;
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufReader, Error as IoError, ErrorKind, Read, Result as IoResult};
use std::path::Path;

pub fn lock_file(lockfile_path: &Path) -> IoResult<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        // Essentially like opening with O_EXCL
        .create_new(true)
        .open(lockfile_path)
        .map(|_| ())
}

pub fn unlock_file(lockfile_path: &Path) -> IoResult<()> {
    if Path::new(&lockfile_path).exists() {
        remove_file(&lockfile_path)
    } else {
        Ok(())
    }
}

// Read a file without locking it
pub fn read_nolock(file_path: &Path, lockfile_path: &Path) -> IoResult<String> {
    if lockfile_path.exists() {
        Err(IoError::new(
            ErrorKind::Other,
            format!("Found lockfile: {}", lockfile_path.display()),
        ))
    } else {
        let mut file_contents = String::new();
        BufReader::new(File::open(file_path)?).read_to_string(&mut file_contents)?;
        Ok(file_contents)
    }
}

// Read a list file and lock it
pub fn read_list_lock(file_path: &Path, lockfile_path: &Path) -> IoResult<Vec<String>> {
    if lockfile_path.exists() {
        Err(IoError::new(
            ErrorKind::Other,
            format!("Found lockfile: {}", lockfile_path.display()),
        ))
    } else {
        lock_file(lockfile_path)?;
        let mut file_contents = String::new();
        BufReader::new(File::open(file_path)?).read_to_string(&mut file_contents)?;
        let lines = file_contents
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>();
        Ok(lines)
    }
}

pub fn error_msg_with_jobs<E: Error>(e: E, msg_start: &str, unwritten_jobs: &[String]) -> String {
    let mut msg = format!("{}\n\n", msg_start);
    msg.push_str("Jobs queued to be written to worktodo:\n");
    for job in unwritten_jobs {
        msg.push_str(job.as_str());
        msg.push('\n');
    }
    msg.push_str("\nPlease add these to your worktodo manually.");
    format!("{}\n\nError: {}", msg, e)
}

pub fn error_msg_with_unwritten<E: Error>(e: E, msg_start: &str, unwritten: &str) -> String {
    let mut msg = format!("{}\n\n", msg_start);
    msg.push_str("Jobs queued to be written to worktodo:\n");
    msg.push_str(unwritten);
    msg.push_str("\nPlease add these to your worktodo manually.");
    format!("{}\n\nError: {}", msg, e)
}
