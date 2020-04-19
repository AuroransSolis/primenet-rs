use crate::{
    clap_handler::{
        app::{GeneralOptions, Gpu72Options, PrimenetOptions},
        gpu72_work::Gpu72WorkType,
        p95_work::PrimenetWorkType,
    },
    primenet_runtime::{primenet_login, primenet_request},
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

const WVR: &str = r"(Factor=N\/A(,[0-9]+){3})$";

fn gpu72_check_login(client: &Client, username: &str, password: &str) -> Result<(), String> {
    let result = client
        .get("https://www.gpu72.com/account/getassignments/")
        .basic_auth(username, Some(password))
        .send()
        .map_err(|e| {
            format!(
                "Failed to send login check request to GPU to 72. Error: {}",
                e
            )
        })?;
    let status = result.status();
    let response = result.text().map_err(|e| {
        format!(
            "Failed to read login check response from GPU to 72. Error: {}",
            e
        )
    })?;
    if status == 200 {
        // Unsure of what a good response looks like, so just print it out and say it's good
        println!("response: '{}'", response);
        Ok(())
    } else {
        Err(format!(
            "Bad response status: {}Error:\n{}",
            status, response
        ))
    }
}

fn gpu72_request(
    client: &Client,
    num_to_cache: usize,
    max_exp: u8,
    worktodo_path: &Path,
    worktodo_lock_path: &Path,
    work_info: Gpu72WorkType,
    username: &str,
    password: &str,
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
        let (worktype_request_addr, workopt) = work_info.as_str();
        let num_to_get = format!("{}", num_to_cache - workfile_contents.len());
        let pledge = format!("{}", max_exp);
        let response = client
            .get(worktype_request_addr)
            .basic_auth(username, Some(password))
            .query(&[
                // Force deref to &str since otherwise &String is expected
                ("Number", &*num_to_get),
                ("GHzDays", ""),
                ("Low", "0"),
                ("High", "10000000000"),
                ("Pledge", &pledge),
                ("Option", workopt),
            ])
            .send()
            .map_err(|e| format!("Failed to make work request to GPU to 72. Error: {}", e))?;
        println!("{:?}", response);
        Ok(())
    }
}

pub fn gpu72_runtime(gpu72_options: Gpu72Options) -> Result<(), String> {
    let Gpu72Options {
        primenet_credentials,
        gpu72_credentials: (gpu72_username, gpu72_password),
        work_type,
        max_exp,
        general_options:
            GeneralOptions {
                work_directory,
                num_cache,
                timeout,
            },
    } = gpu72_options;
    let client = ClientBuilder::default()
        .cookie_store(true)
        .build()
        .map_err(|e| format!("Failed to build web client. Error: {}", e))?;
    if let Some((p95_username, p95_password)) = primenet_credentials {
        primenet_login(&client, &p95_username, &p95_password)?;
        println!("Successfully logged into Primenet.");
    }
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
}
