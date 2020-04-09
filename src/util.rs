use crate::clap_handler::{
    app::{GeneralOptions, Gpu72Options, Options, PrimenetOptions},
    gpu72_work::Gpu72WorkType,
    p95_work::PrimenetWorkType,
};
use regex::Regex;
use reqwest::blocking::{Client, ClientBuilder};
use std::error::Error;
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufReader, BufWriter, Error as IoError, ErrorKind, Read, Result as IoResult, Write};
use std::path::Path;
use std::str::from_utf8;
use std::thread::sleep;
use std::time::{Duration, Instant};

// Work validation regex
const WVR: &str = r"((DoubleCheck|Test|PRP)\s*=\s*([0-9A-F]){32}(,[0-9]+){3}((,-?[0-9]+){3,5})?)$";

const P95_LOGIN_ADDR: &str = "https://www.mersenne.org/default.php";
const P95_REQUEST_ADDR: &str = "https://www.mersenne.org/manual_assignment/";
const GPU72_BASE_ADDR: &str = "https://www.gpu72.com/";

fn lock_file(lockfile_path: &Path) -> IoResult<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        // Essentially like opening with O_EXCL
        .create_new(true)
        .open(lockfile_path)
        .map(|_| ())
}

fn unlock_file(lockfile_path: &Path) -> IoResult<()> {
    if Path::new(&lockfile_path).exists() {
        remove_file(&lockfile_path)
    } else {
        Ok(())
    }
}

// Read a file without locking it
fn read_nolock(file_path: &Path, lockfile_path: &Path) -> IoResult<String> {
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
fn read_list_lock(file_path: &Path, lockfile_path: &Path) -> IoResult<Vec<String>> {
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

fn primenet_login(client: &Client, username: &str, password: &str) -> Result<(), String> {
    let result = client
        .post(P95_LOGIN_ADDR)
        .query(&[("user_login", username), ("user_password", password)])
        .send()
        .map_err(|e| format!("Failed to log in to Primenet. Error: {}", e))?;
    let status = result.status().as_u16();
    if status == 200 {
        let url = result.url().clone();
        let result_text = result
            .text()
            .map_err(|e| format!("Failed to read response text. Error: {}", e))?;
        if result_text.contains(&format!("{}<br>logged in", username)) {
            Ok(())
        } else {
            println!("Failed to log in to Primenet.");
            println!("Login URL: {}", url);
            println!("Request status code: {}", status);
            Err("Login failed.".to_string())
        }
    } else {
        println!(
            "Login attempt at address '{}' returned bad status: {}",
            result.url(),
            status
        );
        Err("Login failed.".to_string())
    }
}

fn error_msg_with_jobs<E: Error>(e: E, msg_start: &str, unwritten_jobs: &[String]) -> String {
    let mut msg = format!("{}\n\n", msg_start);
    msg.push_str("Jobs queued to be written to worktodo:\n");
    for job in unwritten_jobs {
        msg.push_str(job.as_str());
        msg.push('\n');
    }
    msg.push_str("\nPlease add these to your worktodo manually.");
    format!("{}\n\nError: {}", msg, e)
}

fn error_msg_with_unwritten<E: Error>(e: E, msg_start: &str, unwritten: &str) -> String {
    let mut msg = format!("{}\n\n", msg_start);
    msg.push_str("Jobs queued to be written to worktodo:\n");
    msg.push_str(unwritten);
    msg.push_str("\nPlease add these to your worktodo manually.");
    format!("{}\n\nError: {}", msg, e)
}

fn primenet_request(
    client: &Client,
    num_to_cache: usize,
    worktodo_path: &Path,
    worktodo_lock_path: &Path,
    work_info: PrimenetWorkType,
) -> Result<(), String> {
    while worktodo_lock_path.exists() {
        sleep(Duration::from_secs(1));
    }
    let workfile_contents = read_list_lock(worktodo_path, worktodo_lock_path)
        .map_err(|e| format!("Failed to read worktodo file. Error: {}", e))?;
    if num_to_cache <= workfile_contents.len() {
        println!(
            "Already have {} assignment(s) cached of the requested {}. Not requesting more.",
            workfile_contents.len(),
            num_to_cache
        );
        unlock_file(worktodo_lock_path)
            .map_err(|e| format!("Failed to unlock worktodo file. Error: {}", e))
    } else {
        let worktype = format!("{}", work_info.value());
        let num_to_get = format!("{}", num_to_cache - workfile_contents.len());
        let response_text = client
            .post(P95_REQUEST_ADDR)
            .query(&[
                ("cores", "1"),
                ("num_to_get", &num_to_get),
                ("pref", &worktype),
                ("exp_lo", ""),
                ("exp_hi", ""),
            ])
            .send()
            .map_err(|e| format!("Failed to make work request to Primenet. Error: {}", e))?
            .text()
            .map_err(|e| format!("Failed to read response text from Primenet. Error: {}", e))?;
        let work_validation_regex =
            Regex::new(WVR).expect("Failed to build regex for task validation");
        let captured_strings = work_validation_regex
            .captures_iter(&response_text)
            .map(|captures| captures.get(0).map(|m| m.as_str().to_string()))
            .collect::<Vec<_>>();
        let validated_jobs = captured_strings
            .into_iter()
            .filter(|cap| cap.is_some())
            .map(|cap| cap.unwrap())
            .collect::<Vec<_>>();
        // On any errors below until everything is written, show what hasn't yet been written and
        // ask the user to add it themselves.
        let mut list_file = BufWriter::new(
            OpenOptions::new()
                .append(true)
                .open(worktodo_path)
                .map_err(|e| {
                    error_msg_with_jobs(e, "Failed to open worktodo file.", &validated_jobs)
                })?,
        );
        for i in 0..validated_jobs.len() {
            list_file.write_all(&[b'\n']).map_err(|e| {
                error_msg_with_jobs(e, "Failed to write to worktodo file.", &validated_jobs[i..])
            })?;
            list_file
                .write_all(validated_jobs[i].as_bytes())
                .map_err(|e| {
                    error_msg_with_jobs(
                        e,
                        "Failed to write to worktodo file.",
                        &validated_jobs[i..],
                    )
                })?;
        }
        list_file.flush().map_err(|e| {
            error_msg_with_unwritten(
                e,
                "Failed to flush buffered reader to worktodo file.",
                from_utf8(list_file.buffer()).unwrap(),
            )
        })?;
        // Everything should be written to the file now, so we should be safe not to include it in
        // the error message.
        remove_file(worktodo_lock_path).map_err(|e| {
            format!(
                "Failed to remove lockfile after writing new jobs to it. Error: {}",
                e
            )
        })?;
        Ok(())
    }
}

pub fn primenet_runtime(primenet_options: PrimenetOptions) -> Result<(), String> {
    let PrimenetOptions {
        credentials: (username, password),
        work_type,
        general_options:
            GeneralOptions {
                work_directory,
                num_cache,
                timeout,
            },
    } = primenet_options;
    let client = ClientBuilder::default()
        .cookie_store(true)
        .build()
        .map_err(|e| format!("Failed to build web client. Error: {}", e))?;
    primenet_login(&client, &username, &password)?;
    let worktodo_txt_path = Path::new(&work_directory).join(Path::new("worktodo.txt"));
    let worktodo_ini_path = Path::new(&work_directory).join(Path::new("worktodo.ini"));
    let (worktodo_path, worktodo_lock_path) = if worktodo_txt_path.exists() {
        (
            worktodo_txt_path,
            Path::new(&work_directory).join(Path::new("worktodo.txt.lck")),
        )
    } else {
        (
            worktodo_ini_path,
            Path::new(&work_directory).join(Path::new("worktodo.ini.lck")),
        )
    };
    if timeout == 0 {
        primenet_request(
            &client,
            num_cache,
            &worktodo_path,
            &worktodo_lock_path,
            work_type,
        )?;
    } else {
        loop {
            let start = Instant::now();
            if let Err(e) = primenet_request(
                &client,
                num_cache,
                &worktodo_path,
                &worktodo_lock_path,
                work_type,
            ) {
                println!("{}", e);
            } else {
                println!("Successfully requested and cached jobs.");
            }
            let sleep_duration = Duration::from_secs(timeout as u64) - start.elapsed();
            sleep(sleep_duration);
        }
    }
    Ok(())
}
