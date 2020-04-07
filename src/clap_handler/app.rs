use super::gpu72_work::*;
use super::p95_work::*;
use super::validators::*;
use clap::{App, Arg, ArgGroup, ArgMatches};
use std::env::current_dir;
use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

pub struct GeneralOptions {
    work_directory: String,
    num_cache: usize,
    timeout: usize,
}

pub struct PrimenetOptions {
    credentials: (String, String),
    work_type: PrimenetWorkType,
    general_options: GeneralOptions,
}

pub struct Gpu72Options {
    primenet_credentials: Option<(String, String)>,
    gpu72_credentials: (String, String),
    fallback: bool,
    work_type: Gpu72WorkType,
    general_options: GeneralOptions,
}

pub enum Options {
    Primenet(PrimenetOptions),
    Gpu72(Gpu72Options),
}

fn request_from_args() -> Result<Options, String> {
    let current_dir = format!("{}", current_dir().unwrap().display());
    let matches = App::new("primenet-rs")
        .version("1.0.0")
        .about("Interface to request from and report to Primenet (GIMPS).")
        .author("Aurorans Solis")
        .arg(
            Arg::with_name("work-directory")
                .short("w")
                .long("work-directory")
                .takes_value(true)
                .number_of_values(1)
                .value_name("WORKDIR")
                .default_value(&current_dir)
                .validator(directory_validator)
                .help(&format!(
                    "Working directory with worktodo.txt/worktodo.ini and results.txt. Default: {}",
                    current_dir
                )),
        )
        .arg(
            Arg::with_name("num-cache")
                .short("n")
                .long("num-cache")
                .takes_value(true)
                .number_of_values(1)
                .value_name("NUM_CACHE")
                .default_value("1")
                .validator(numeric_validator)
                .help("Number of assignments to cache. Default: 1")
                .required(true),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .number_of_values(1)
                .value_name("TIMEOUT")
                .default_value("0")
                .validator(numeric_validator)
                .help(
                    "Seconds to wait between network updates. Use 0 for a single update without \
                    looping.",
                )
                .required(true),
        )
        .group(
            ArgGroup::with_name("general options")
                .args(&["work-directory", "num-cache", "timeout"])
                .multiple(true),
        )
        .arg(
            Arg::with_name("p95-username")
                .long("p95-username")
                .takes_value(true)
                .number_of_values(1)
                .value_name("USERNAME")
                .validator(p95_username_validator)
                .help("Primenet username"),
        )
        .arg(
            Arg::with_name("p95-password")
                .long("p95-password")
                .takes_value(true)
                .number_of_values(1)
                .value_name("PASSWORD")
                .help("Primenet password"),
        )
        .group(
            ArgGroup::with_name("p95-userpass")
                .args(&["p95-username", "p95-password"])
                .multiple(true),
        )
        .arg(
            Arg::with_name("p95-username-file")
                .long("p95-username-file")
                .takes_value(true)
                .number_of_values(1)
                .value_name("FILE_PATH")
                .validator(file_validator)
                .help("Path to file containing Primenet username"),
        )
        .arg(
            Arg::with_name("p95-password-file")
                .long("p95-password-file")
                .takes_value(true)
                .number_of_values(1)
                .value_name("FILE_PATH")
                .validator(file_validator)
                .help("Path to file containing Primenet password"),
        )
        .group(
            ArgGroup::with_name("p95-userpass-files")
                .args(&["p95-username-file", "p95-password-file"])
                .multiple(true),
        )
        .group(
            ArgGroup::with_name("p95-credentials")
                .args(&[
                    "p95-userpass",
                    "p95-userpass-files",
                ])
                .multiple(false),
        )
        .arg(
            Arg::with_name("p95-trial-factoring")
                .short("p95-tf")
                .long("p95-trial-factoring")
                .help("Request trial factoring work from Primenet"),
        )
        .arg(
            Arg::with_name("p95-p1-factoring")
                .short("p95-p1")
                .long("p95-p1-factoring")
                .help("Request P-1 factoring work from Primenet"),
        )
        .arg(
            Arg::with_name("p95-optimal-p1-factoring")
                .short("p95-op1")
                .long("p95-optimal-p1-factoring")
                .help("Request optimal P-1 factoring work from Primenet"),
        )
        .arg(
            Arg::with_name("p95-ecm-factoring")
                .short("p95-ecm")
                .long("p95-ecm-factoring")
                .help("Request ECM factoring work from Primenet"),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-first-time")
                .short("p95-llft")
                .long("p95-lucas-lehmer-first-time")
                .help("Request LL first time work from Primenet"),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-double-check")
                .short("p95-lldc")
                .long("p95-lucas-lehmer-double-check")
                .help("Request LL double-check work from Primenet"),
        )
        .group(
            ArgGroup::with_name("p95-worktypes")
                .args(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ])
                .multiple(false)
                .requires("p95-workopts"),
        )
        .arg(
            Arg::with_name("p95-what-makes-most-sense")
                .short("p95-wmms")
                .long("p95-what-makes-most-sense")
                .help("Ask Primenet to assign whatever makes most sense"),
        )
        .arg(
            Arg::with_name("p95-factoring-lmh")
                .short("p95-flmh")
                .long("p95-factoring-lmh")
                .help("Request factoring LFH work from Primenet")
                .conflicts_with_all(&[
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-factoring-trial-sieve")
                .short("p95-fts")
                .long("p95-factoring-trial-seve")
                .help("Request factoring trail (sieve) work from Primenet")
                .conflicts_with_all(&[
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-factoring-p1-small")
                .short("p95-fp1s")
                .long("p95-factoring-p1-small")
                .help("Request small P-1 factoring work from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-factoring-p1-large")
                .short("p95-fp1l")
                .long("p95-factoring-p1-large")
                .help("Request large P-1 factoring work from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-smallish-ecm")
                .short("p95-secm")
                .long("p95-smallish-ecm")
                .help("Request smallish ECM factoring work from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-fermat-ecm")
                .short("p95-fecm")
                .long("p95-fermat-ecm")
                .help("Request Fermat ECM factoring work from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-cunningham-ecm")
                .short("p95-cecm")
                .long("p95-cunningham-ecm")
                .help("Request Cunningham ECM factoring work from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-lucas-lehmer-first-time",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-first-time-test")
                .short("p95-ll-ft")
                .long("p95-lucas-lehmer-first-time-test")
                .help("Request LL first time tests from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-double-check")
                .short("p95-ll-dc")
                .long("p95-lucas-lehmer-double-check")
                .help("Request LL double-check tests from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-first-time",
                ]),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-world-record")
                .short("p95-ll-wr")
                .long("p95-lucas-lehmer-first-time-test")
                .help("Request LL world record tests from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-10m-digit")
                .short("p95-ll-10md")
                .long("p95-lucas-lehmer-10m-digit")
                .help("Request LL 10M digit tests from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-100m-digit")
                .short("p95-ll-100md")
                .long("p95-lucas-lehmer-100m-digit")
                .help("Request LL 100M digit tests from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .arg(
            Arg::with_name("p95-lucas-lehmer-no-trial-or-p1")
                .short("p95-ll-ntop1")
                .long("p95-lucas-lehmer-no-trial-or-p1")
                .help("Request LL first time tests with no trial or P-1 factoring from Primenet")
                .conflicts_with_all(&[
                    "p95-trial-factoring",
                    "p95-p1-factoring",
                    "p95-optimal-p1-factoring",
                    "p95-ecm-factoring",
                    "p95-lucas-lehmer-double-check",
                ]),
        )
        .group(
            ArgGroup::with_name("p95-workopts")
                .args(&[
                    "p95-what-makes-most-sense",
                    "p95-factoring-lmh",
                    "p95-factoring-trial-sieve",
                    "p95-factoring-p1-small",
                    "p95-factoring-p1-large",
                    "p95-smallish-ecm",
                    "p95-fermat-ecm",
                    "p95-cunningham-ecm",
                    "p95-lucas-lehmer-first-time-test",
                    "p95-lucas-lehmer-double-check",
                    "p95-lucas-lehmer-world-record",
                    "p95-lucas-lehmer-10m-digit",
                    "p95-lucas-lehmer-100m-digit",
                    "p95-lucas-lehmer-no-trial-or-p1",
                ])
                .multiple(false),
        )
        .group(
            ArgGroup::with_name("p95-work")
                .args(&["p95-worktypes", "p95-workopts"])
        )
        .arg(
            Arg::with_name("gpu72-username")
                .long("gpu72-username")
                .takes_value(true)
                .number_of_values(1)
                .value_name("USERNAME")
                .validator(gpu72_username_validator)
                .help("GPU to 72 username"),
        )
        .arg(
            Arg::with_name("gpu72-password")
                .long("gpu72-password")
                .takes_value(true)
                .number_of_values(1)
                .value_name("PASSWORD")
                .help("GPU to 72 password"),
        )
        .group(
            ArgGroup::with_name("gpu72-userpass")
                .args(&["gpu72-username", "gpu72-password"])
                .multiple(true),
        )
        .arg(
            Arg::with_name("gpu72-username-file")
                .long("gpu72-username-file")
                .takes_value(true)
                .number_of_values(1)
                .value_name("FILE_PATH")
                .validator(file_validator)
                .help("Path to file containing GPU to 72 username"),
        )
        .arg(
            Arg::with_name("gpu72-password-file")
                .long("gpu72-password-file")
                .takes_value(true)
                .number_of_values(1)
                .value_name("FILE_PATH")
                .validator(file_validator)
                .help("Path to file containing GPU to 72 password"),
        )
        .group(
            ArgGroup::with_name("gpu72-userpass-files")
                .args(&["gpu72-username-file", "gpu72-password-file"])
                .multiple(true),
        )
        .group(
            ArgGroup::with_name("gpu72-credentials")
                .args(&[
                    "gpu72-userpass",
                    "gpu72-userpass-files",
                ])
                .multiple(false)
                .requires("gpu72-work"),
        )
        .arg(
            Arg::with_name("gpu72-fallback")
                .long("gpu72-fallback")
                .help("Fall back to Primenet if requests to GPU to 72 fail or it has no work")
                .requires("p95-credentials"),
        )
        .arg(
            Arg::with_name("gpu72-lucas-lehmer-trial-factor")
                .short("gpu72-lltf")
                .long("gpu72-lucas-lehmer-trial-factor")
                .help("Request LL trial factoring work from GPU to 72")
        )
        .arg(
            Arg::with_name("gpu72-double-check-trial-factor")
                .short("gpu72-dctf")
                .long("gpu72-double-check-trial-factor")
                .help("Request double-check trial factoring work from GPU to 72")
        )
        .arg(
            Arg::with_name("gpu72-lucas-lehmer-p1")
                .short("gpu72-llp1")
                .long("gpu72-lucas-lehmer-p1")
                .help("Request LL P-1 work from GPU to 72")
        )
        .arg(
            Arg::with_name("gpu72-double-check-p1")
                .short("gpu72-dcp1")
                .long("gpu72-double-check-p1")
                .takes_value(true)
                .number_of_values(1)
                .value_name("EFFORT")
                .default_value("2.0")
                .validator(f32_validator)
                .help("Request double-check P-1 ork from GPU to 72. Note: effort below 1.0 is pointless.")
        )
        .group(
            ArgGroup::with_name("gpu72-worktype-require-opts")
                .args(&[
                    "gpu72-lucas-lehmer-trial-factor",
                    "gpu72-double-check-trial-factor",
                    "gpu72-lucas-lehmer-p1",
                ])
                .multiple(false)
                .requires("gpu72-workopts")
        )
        .group(
            ArgGroup::with_name("gpu72-worktypes")
                .args(&["gpu72-worktype-require-opts", "gpu72-double-check-p1"])
                .multiple(false)
        )
        .arg(
            Arg::with_name("gpu72-what-makes-most-sense")
                .short("gpu72-wmms")
                .long("gpu72-what-makes-most-sense")
                .help("Ask GPU to 72 to assign whatever makes most sense.")
                .conflicts_with("gpu72-double-check-p1")
        )
        .arg(
            Arg::with_name("gpu72-lowest-trial-factor-level")
                .short("gpu72-ltfl")
                .long("gpu72-lowest-trial-factor-level")
                .help("Request work of the lowest trial factoring level from GPU to 72")
                .conflicts_with_all(&["gpu72-lucas-lehmer-p1", "gpu72-double-check-p1"])
        )
        .arg(
            Arg::with_name("gpu72-highest-trial-factor-level")
                .short("gpu72-htfl")
                .long("gpu72-highest-trial-factor-level")
                .help("Request work of the highest trial factoring level from GPU to 72")
                .conflicts_with_all(&["gpu72-lucas-lehmer-p1", "gpu72-double-check-p1"])
        )
        .arg(
            Arg::with_name("gpu72-lowest-exponent")
                .short("gpu72-le")
                .long("gpu72-lowest-exponent")
                .help("Request the lowest exponent for the selected work type from GPU to 72")
                .conflicts_with("gpu72-double-check-p1")
        )
        .arg(
            Arg::with_name("gpu72-oldest-exponent")
                .short("gpu72-oe")
                .long("gpu72-oldest-exponent")
                .help("Request the oldest exponent for the selected work type from GPU to 72")
                .conflicts_with("gpu72-double-check-p1")
        )
        .arg(
            Arg::with_name("gpu72-double-check-already-done")
                .short("gpu72-dcad")
                .long("gpu72-double-check-already-done")
                .help(
                    "Request double-check trial factoring work where a double check has already \
                    been done from GPU to 72"
                )
                .conflicts_with_all(&[
                    "gpu72-lucas-lehmer-trial-factor",
                    "gpu72-lucas-lehmer-p1",
                    "gpu72-double-check-p1",
                ])
        )
        .arg(
            Arg::with_name("gpu72-lone-mersenne-hunters-bit-first")
                .short("gpu72-lmh-bf")
                .long("gpu72-lone-mersenne-hunters-bit-first")
                .help("Request LMH bit-first work from GPU to 72")
                .conflicts_with_all(&[
                    "gpu72-double-check-trial-factor",
                    "gpu72-lucas-lehmer-p1",
                    "gpu72-double-check-p1",
                ])
        )
        .arg(
            Arg::with_name("gpu72-lone-mersenne-hunters-depth-first")
                .short("gpu72-lmh-df")
                .long("gpu72-lone-mersenne-hunters-depth-first")
                .help("Request LMH depth-first work from GPU to 72")
                .conflicts_with_all(&[
                    "gpu72-double-check-trial-factor",
                    "gpu72-lucas-lehmer-p1",
                    "gpu72-double-check-p1",
                ])
        )
        .arg(
            Arg::with_name("gpu72-let-gpu72-decide")
                .short("gpu72-lgpu72d")
                .long("gpu72-let-gpu72-decide")
                .help("Let GPU to 72 decide what type of work to do.")
                .conflicts_with_all(&["gpu72-lucas-lehmer-p1", "gpu72-double-check-p1"])
        )
        .group(
            ArgGroup::with_name("gpu72-workopts")
                .args(&[
                    "gpu72-what-makes-sense",
                    "gpu72-lowest-trial-factor-level",
                    "gpu72-highest-trial-factor-level",
                    "gpu72-lowest-exponent",
                    "gpu72-oldest-exponent",
                    "gpu72-double-check-already-done",
                    "gpu72-lone-mersenne-hunters-bit-first",
                    "gpu72-lone-mersenne-hunters-depth-first",
                    "gpu72-let-gpu72-decide"
                ])
                .multiple(false)
        )
        .group(
            ArgGroup::with_name("gpu72-work")
                .args(&["gpu72-worktypes", "gpu72-workopts"])
                .multiple(false)
                .conflicts_with("p95-work")
        ).get_matches_safe().map_err(|e| format!("{}", e))?;
    if matches.is_present("gpu72-credentials") {
        let gpu72_credentials = if matches.is_present("gpu72-userpass") {
            (
                matches.value_of("gpu72-username").unwrap().to_string(),
                matches.value_of("gpu72-password").unwrap().to_string(),
            )
        } else {
            let username_path = matches.value_of("gpu72-username-file").unwrap();
            let mut username_file = BufReader::new(File::open(username_path).unwrap());
            let mut username = String::new();
            username_file.read_to_string(&mut username);
            let password_path = matches.value_of("gpu72-password-file").unwrap();
            let mut password_file = BufReader::new(File::open(password_path).unwrap());
            let mut password = String::new();
            password_file.read_to_string(&mut password);
            (username, password)
        };
        let fallback = matches.is_present("gpu72-fallback");
        let primenet_credentials = if fallback {
            if matches.is_present("p95-userpass") {
                Some((
                    matches.value_of("p95-username").unwrap().to_string(),
                    matches.value_of("p95-password").unwrap().to_string(),
                ))
            } else {
                let username_path = matches.value_of("p95-username-file").unwrap();
                let mut username_file = BufReader::new(File::open(username_path).unwrap());
                let mut username = String::new();
                username_file.read_to_string(&mut username);
                let password_path = matches.value_of("p95-password-file").unwrap();
                let mut password_file = BufReader::new(File::open(password_path).unwrap());
                let mut password = String::new();
                password_file.read_to_string(&mut password);
                Some((username, password))
            }
        } else {
            None
        };
        let work_directory = matches.value_of("work-directory").unwrap().to_string();
        let num_cache = matches
            .value_of("num-cache")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let timeout = matches.value_of("timeout").unwrap().parse::<usize>().unwrap();
        let general_options = GeneralOptions {
            work_directory,
            num_cache,
            timeout,
        };
        let work_type = if matches.is_present("gpu72-lucas-lehmer-trial-factor") {
            Gpu72WorkType::LucasLehmerTrialFactor(if matches.is_present("gpu72-what-makes-sense") {
                Gpu72LLTFWorkOption::WhatMakesSense
            } else if matches.is_present("gpu72-lowest-trial-factor-level") {
                Gpu72LLTFWorkOption::LowestTrialFactorLevel
            } else if matches.is_present("gpu72-highest-trial-factor-level") {
                Gpu72LLTFWorkOption::HighestTrialFactorLevel
            } else if matches.is_present("gpu72-lowest-exponent") {
                Gpu72LLTFWorkOption::LowestExponent
            } else if matches.is_present("gpu72-oldest-exponent") {
                Gpu72LLTFWorkOption::OldestExponent
            } else if matches.is_present("gpu72-lone-mersenne-hunters-bit-first") {
                Gpu72LLTFWorkOption::LoneMersenneHuntersBitFirst
            } else if matches.is_present("gpu72-lone-mersenne-hunters-depth-first") {
                Gpu72LLTFWorkOption::LoneMersenneHuntersDepthFirst
            } else {
                Gpu72LLTFWorkOption::LetGpu72Decide
            })
        } else if matches.is_present("gpu72-double-check-trial-factor") {
            Gpu72WorkType::DoubleCheckTrialFactor(if matches.is_present("gpu72-what-makes-sense") {
                Gpu72DCTFWorkOption::WhatMakesSense
            } else if matches.is_present("gpu72-lowest-trial-factor-level") {
                Gpu72DCTFWorkOption::LowestTrialFactorLevel
            } else if matches.is_present("gpu72-highest-trial-factor-level") {
                Gpu72DCTFWorkOption::HighestTrialFactorLevel
            } else if matches.is_present("gpu72-lowest-exponent") {
                Gpu72DCTFWorkOption::LowestExponent
            } else if matches.is_present("gpu72-oldest-exponent") {
                Gpu72DCTFWorkOption::OldestExponent
            } else if matches.is_present("gpu72-double-check-already-done") {
                Gpu72DCTFWorkOption::DoubleCheckAlreadyDone
            } else {
                Gpu72DCTFWorkOption::LetGpu72Decide
            })
        } else if matches.is_present("gpu72-lucas-lehmer-p1") {
            Gpu72WorkType::LucasLehmerP1(if matches.is_present("gpu72-lowest-exponent") {
                Gpu72LLP1WorkOption::LowestExponent
            } else if matches.is_present("gpu72-oldest-exponent") {
                Gpu72LLP1WorkOption::OldestExponent
            } else {
                Gpu72LLP1WorkOption::WhatMakesSense
            })
        } else {
            Gpu72WorkType::DoubleCheckP1(matches.value_of("gpu72-double-check-p1").unwrap().parse::<f32>().unwrap())
        };
        Ok(Options::Gpu72(Gpu72Options {
            primenet_credentials,
            gpu72_credentials,
            fallback,
            work_type,
            general_options,
        }))
    } else if matches.is_present("p95-credentials") {
        Err("".to_string())
    } else {
        Err(
            "Missing minimum requirements: \n    \
                - Primenet login credentials, Primenet work type, Primenet work options\n    \
                - GPU to 72 login credentials, GPU to 72 work type, GPU to 72 work type\n        \
                        if '--gpu72-fallback' is passed in, also include Primenet login credentials"
            .to_string()
        )
    }
}