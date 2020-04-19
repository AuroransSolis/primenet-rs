use crate::{
    clap_handler::{
        app::{GeneralOptions, PrimenetOptions},
        p95_work::PrimenetWorkType,
    },
    util::*,
};
use regex::RegexBuilder;
use reqwest::blocking::{Client, ClientBuilder};
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::str::from_utf8;
use std::thread::sleep;
use std::time::{Duration, Instant};

// Work validation regex
const WVR: &str = r"((DoubleCheck|Test|PRP)\s*=\s*([0-9A-F]){32}(,[0-9]+){3}((,-?[0-9]+){3,5})?)$";

const P95_LOGIN_ADDR: &str = "https://www.mersenne.org/";
const P95_REQUEST_ADDR: &str = "https://www.mersenne.org/manual_assignment/?";
const P95_REPORT_ADDR: &str = "https://www.mersenne.org/manual_result/?";

pub fn primenet_login(client: &Client, username: &str, password: &str) -> Result<(), String> {
    let result = client
        .post(P95_LOGIN_ADDR)
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
        let response = client
            .get(P95_REQUEST_ADDR)
            .query(&[
                ("cores", "1"),
                ("num_to_get", &num_to_get),
                ("pref", worktype),
                ("exp_lo", ""),
                ("exp_hi", ""),
                ("B1", "Get+Assignments"),
            ])
            .send()
            .map_err(|e| format!("Failed to make work request to Primenet. Error: {}", e))?;
        let status = response.status().as_u16();
        let response_text = response
            .text()
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
            if validated_jobs.len() == 0 {
                println!("WARNING!");
                println!(
                    "Received work request response but failed to find any valid jobs in it. You \n\
                    may want to check your Primenet account to see if any work has been \n\
                    reserved, and if so, add it to your worktodo file manually."
                );
                return Ok(());
            }
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
        } else {
            println!("Failed to request work from Primenet. Status: {}", status);
            println!("Response text: {}", response_text);
        }
        Ok(())
    }
}

fn writeback_on_failure(results_bufwriter: &mut BufWriter<File>, unsent_result: String) {
    // If submission fails, write the result back to the results file.
}

pub fn primenet_submit(
    client: &Client,
    worktodo_path: &Path,
    worktodo_lock_path: &Path,
    results_path: &Path,
    results_lock_path: &Path,
    results_sent_path: &Path,
    results_sent_lock_path: &Path,
) -> Result<(), String> {
    let worktodo_contents = read_list_lock(worktodo_path, worktodo_lock_path)
        .map_err(|e| format!("Could not lock and read worktodo file. Error: {}", e))?;
    let mut results_contents = read_list_lock(results_path, results_lock_path)
        .map_err(|e| format!("Could not lock and read results file. Error: {}", e))?;
    lock_file(results_sent_lock_path)?;
    let mut results_file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .append(false)
            .open(results_path)
            .map_err(|e| {
                format!(
                    "Failed to open results file with write privileges. Error: {}",
                    e
                )
            })?,
    );
    let mut results_sent_file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .append(true)
            .open(results_sent_path)
            .map_err(|e| {
                format!(
                    "Failed to open sent results file with write privileges. Error: {}",
                    e
                )
            })?,
    );
    let mut collisions = Vec::new();
    // Only jobs that are completed are allowed to be submitted.
    // Could handle this with hashset collisions, but then I have to sort out ordering when writing
    // back to the file on submission errors. Resolve this.
    for job in &worktodo_contents {
        if let Some(pos) = results_contents.iter().position(|j| j == job) {
            collisions.push(results_contents.remove(pos));
        }
    }
    println!("Found the following incomplete jobs in results.txt:");
    for collision in collisions {
        println!("    {}", collision);
    }
    if results_contents.len() > 0 {
        while let Some(completed_job) = results_contents.pop() {
            let response_text = client
                .post(P95_REPORT_ADDR)
                .form(&[("data", "completed_job")])
                .send()
                .map_err(|e| format!("Failed to send work submission to primenet. Error: {}", e))?
                .text()
                .map_err(|e| {
                    format!(
                        "Failed to read response text from work submission to Primenet. Error: {}",
                        e
                    )
                })?;
            if response_text.contains("Error") {
                let e_start = response_text.find("Error").unwrap();
                let e_end = response_text[e_start..].find("</div>").unwrap();
                println!(
                    "Submission failed. Error message from Primenet: {}",
                    &response_text[e_start..e_end]
                );
                writeback_on_failure(&mut results_file, completed_job);
            } else if response_text.contains("Accepted") {
                // Submission was accepted by Primenet - write result to results.sent.txt
            } else {
                // Unknown failure case - write failed submission back to results.txt
            }
        }
    }
    unlock_file(worktodo_lock_path).map_err(|e| {
        format!(
            "Could not remove lockfile {}. Error: {}",
            worktodo_lock_path.display(),
            e
        )
    })?;
    unlock_file(results_lock_path).map_err(|e| {
        format!(
            "Could not remove lockfile {}. Error: {}",
            results_lock_path.display(),
            e
        )
    })?;
    unlock_file(results_sent_lock_path).map_err(|e| {
        format!(
            "Could not remove lockfile {}. Error: {}",
            results_sent_lock_path.display(),
            e
        )
    })?;
    Ok(())
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
        (worktodo_txt_path, worktodo_txt_path.join(".lck"))
    } else {
        (worktodo_ini_path, worktodo_ini_path.join(".lck"))
    };
    let results_path = Path::new(&work_directory).join(Path::new("results.txt"));
    let results_lock_path = results_path.join(".lck");
    let results_sent_path = Path::new(&work_directory).join(Path::new("results.sent"));
    let results_sent_lock_path = results_sent_path.join(".lck");
    println!("Using worktodo path: {}", worktodo_path.display());
    println!("Using worktodo_lock path: {}", worktodo_lock_path.display());
    println!("Using results path: {}", results_path.display());
    if timeout == 0 {
        primenet_request(
            &client,
            num_cache,
            &worktodo_path,
            &worktodo_lock_path,
            work_type,
        )?;
        primenet_submit(
            &client,
            &worktodo_path,
            &worktodo_lock_path,
            &results_path,
            &results_lock_path,
            &results_sent_path,
            &results_sent_lock_path,
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
            if let Err(e) = primenet_submit(
                &client,
                &worktodo_path,
                &worktodo_lock_path,
                &results_path,
                &results_lock_path,
                &results_sent_path,
                &results_sent_lock_path,
            ) {
                println!("{}", e);
            } else {
                println!(
                    "Successfully submitted cached results to Primenet. Submitted results can be"
                );
                println!("found in $WORDKDIR/results.sent until next submission.");
            }
            let sleep_duration = Duration::from_secs(timeout as u64) - start.elapsed();
            sleep(sleep_duration);
        }
    }
    Ok(())
}

pub fn primenet_cleanup(primenet_options: PrimenetOptions) {}
