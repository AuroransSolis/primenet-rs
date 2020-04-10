use crate::{
    clap_handler::{
        app::{GeneralOptions, PrimenetOptions},
        p95_work::PrimenetWorkType,
    },
    util::*,
};
use regex::RegexBuilder;
use reqwest::blocking::{Client, ClientBuilder};
use std::fs::{remove_file, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::str::from_utf8;
use std::thread::sleep;
use std::time::{Duration, Instant};

// Work validation regex
const WVR: &str = r"((DoubleCheck|Test|PRP)\s*=\s*([0-9A-F]){32}(,[0-9]+){3}((,-?[0-9]+){3,5})?)$";

const P95_LOGIN_ADDR: &str = "https://www.mersenne.org/";
const P95_REQUEST_ADDR: &str = "https://www.mersenne.org/manual_assignment/?";

fn primenet_login(client: &Client, username: &str, password: &str) -> Result<(), String> {
    let result = client.post(P95_LOGIN_ADDR)
        .form(&[("user_login", username), ("user_password", password)])
        .send()
        .map_err(|e| format!("Failed to send login attempt to Primenet. Error: {}", e))?;
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
            println!("Login response: {}", result_text);
            Err("Login failed.".to_string())
        }
    } else {
        println!(
            "Login attempt at address '{}' returned bad status: {}",
            result.url(),
            status
        );
        println!("Failure response: {:?}", result.text());
        Err("Login failed.".to_string())
    }
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
        let worktype = work_info.as_str();
        let num_to_get = format!("{}", num_to_cache - workfile_contents.len());
        // Formatting the request address myself since reqwest can't figure out how to do it right
        // or something. Great job.
        let request_address = format!(
            "{}cores=1&num_to_get={}&pref={}&exp_lo=&exp_hi=&B1=Get%2BAssignments",
            P95_REQUEST_ADDR, num_to_get, worktype
        );
        let response = client
            .get(&request_address)
            .send()
            .map_err(|e| format!("Failed to make work request to Primenet. Error: {}", e))?;
        let status = response.status().as_u16();
        let response_text = response.text()
            .map_err(|e| format!("Failed to read response text from Primenet. Error: {}", e))?;
        if status == 200 {
            println!("Got work request response.");
            let work_validation_regex = RegexBuilder::new(WVR)
                .multi_line(true)
                .build()
                .expect("Failed to build regex for task validation");
            let validated_jobs = work_validation_regex
                .captures_iter(&response_text)
                .map(|captures| captures[0].to_string())
                .collect::<Vec<_>>();
            println!("Validated jobs: {:?}", validated_jobs);
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
                list_file
                    .write_all(validated_jobs[i].as_bytes())
                    .map_err(|e| {
                        error_msg_with_jobs(
                            e,
                            "Failed to write to worktodo file.",
                            &validated_jobs[i..],
                        )
                    })?;
                list_file.write_all(&[b'\n']).map_err(|e| {
                    error_msg_with_jobs(e, "Failed to write to worktodo file.", &validated_jobs[i..])
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
        } else {
            println!("Failed to request work from Primenet. Status: {}", status);
            println!("Response text: {}", response_text);
        }
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
    println!("Successfully logged into Primenet.");
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
    let results = Path::new(&work_directory).join(Path::new("results.txt"));
    println!("Using worktodo path: {}", worktodo_path.display());
    println!("Using worktodo_lock path: {}", worktodo_lock_path.display());
    println!("Using results path: {}", results.display());
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