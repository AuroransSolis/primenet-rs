use crate::clap_handler::{app::Options, gpu72_work::Gpu72WorkType, p95_work::PrimenetWorkType};
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufReader, BufWriter, Error as IoError, ErrorKind, Read, Result as IoResult, Write};
use std::path::Path;

fn lock_file(filename: &str) -> IoResult<()> {
    let lockfile_name = format!("{}.lck", filename);
    OpenOptions::new()
        .read(true)
        .write(true)
        // Essentially like opening with O_EXCL
        .create_new(true)
        .open(&lockfile_name)
        .map(|_| ())
}

fn unlock_file(filename: &str) -> IoResult<()> {
    let lockfile_name = format!("{}.lck", filename);
    if Path::new(&lockfile_name).exists() {
        remove_file(&lockfile_name)?;
    }
    Ok(())
}

// Read a file without locking it
fn read_nolock(filename: &str) -> IoResult<String> {
    let lockfile_name = format!("{}.lck", filename);
    if Path::new(&lockfile_name).exists() {
        Err(IoError::new(
            ErrorKind::Other,
            format!("Found lockfile: {}", lockfile_name),
        ))
    } else {
        let mut file_contents = String::new();
        BufReader::new(File::open(filename)?).read_to_string(&mut file_contents)?;
        Ok(file_contents)
    }
}

// Read a list file and lock it
fn read_list_lock(filename: &str) -> IoResult<Vec<String>> {
    let lockfile_name = format!("{}.lck", filename);
    if Path::new(&lockfile_name).exists() {
        Err(IoError::new(
            ErrorKind::Other,
            format!("Found lockfile: {}", lockfile_name),
        ))
    } else {
        let _ = File::create(&lockfile_name)?;
        let mut file_contents = String::new();
        BufReader::new(File::open(filename)?).read_to_string(&mut file_contents)?;
        let lines = file_contents
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>();
        Ok(lines)
    }
}

// Write a list to a file
fn write_list(filename: &str, contents: &str) -> IoResult<()> {
    let lockfile_name = format!("{}.lck", filename);
    if Path::new(&lockfile_name).exists() {
        Err(IoError::new(
            ErrorKind::Other,
            format!("Found lockfile: {}", lockfile_name),
        ))
    } else {
        let _ = File::create(&lockfile_name)?;
        let mut list_file = BufWriter::new(File::open(filename)?);
        list_file.write_all(&[b'\n'])?;
        list_file.write_all(contents.as_bytes())?;
        list_file.write_all(&[b'\n'])?;
        remove_file(&lockfile_name)?;
        Ok(())
    }
}

fn primenet_fetch(amt_to_get: usize, options: Options) /*->*/ {}
