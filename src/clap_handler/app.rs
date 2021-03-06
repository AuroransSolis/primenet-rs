use super::gpu72_work::*;
use super::lists::*;
use super::p95_work::*;
use super::validators::*;
use clap::{App, Arg, ArgGroup};
use std::env::current_dir;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Clone, Debug)]
pub struct GeneralOptions {
    pub work_directory: String,
    pub num_cache: usize,
    pub timeout: usize,
}

#[derive(Clone, Debug)]
pub struct PrimenetOptions {
    pub credentials: (String, String),
    pub work_type: PrimenetWorkType,
    pub general_options: GeneralOptions,
}

#[derive(Clone, Debug)]
pub struct Gpu72Options {
    pub primenet_credentials: Option<(String, String)>,
    pub gpu72_credentials: (String, String),
    pub work_type: Gpu72WorkType,
    pub max_exp: u8,
    pub general_options: GeneralOptions,
}

#[derive(Clone, Debug)]
pub enum Options {
    Primenet(PrimenetOptions),
    Gpu72(Gpu72Options),
}

macro_rules! map_matches {
    (
        $matches:ident,
        $work_string_i:literal => $worktype_i:path {
            $($worktype_i_opt_string_i:literal -> $worktype_i_opt_i:expr;
            $(
                $worktype_i_opt_string_ei:literal -> $worktype_i_opt_ei:expr;
            )*
            _ -> $worktype_i_opt_e:expr;)?
            $(_ -> $worktype_i_other:expr;)?
        }
        $($work_string_ei:literal => $worktype_ei:path {
            $($worktype_ei_opt_string_i:literal -> $worktype_ei_opt_i:expr;
            $(
                $worktype_ei_opt_string_ei:literal -> $worktype_ei_opt_ei:expr;
            )*
            _ -> $worktype_ei_opt_e:expr;)?
            $(_ -> $worktype_ei_other:expr;)?
        })*
        _ => $worktype_e:path {
            $($worktype_e_opt_string_i:literal -> $worktype_e_opt_i:expr;
            $(
                $worktype_e_opt_string_ei:literal -> $worktype_e_opt_ei:expr;
            )*
            _ -> $worktype_e_opt_e:expr;)?
            $(_ -> $worktype_e_other:expr;)?
        }
    ) => {{
        if $matches.is_present($work_string_i) {
            $($worktype_i(if $matches.is_present($worktype_i_opt_string_i) {
                $worktype_i_opt_i
            } $(else if $matches.is_present($worktype_i_opt_string_ei) {
                $worktype_i_opt_ei
            })* else {
                $worktype_i_opt_e
            }))?
            $($worktype_i_other)?
        } $(else if $matches.is_present($work_string_ei) {
            $($worktype_ei(if $matches.is_present($worktype_ei_opt_string_i) {
                $worktype_ei_opt_i
            } $(else if $matches.is_present($worktype_ei_opt_string_ei) {
                $worktype_ei_opt_ei
            })* else {
                $worktype_ei_opt_e
            }))?
            $($worktype_ei_other)?
        })* else {
            $($worktype_e(if $matches.is_present($worktype_e_opt_string_i) {
                $worktype_e_opt_i
            } $(else if $matches.is_present($worktype_e_opt_string_ei) {
                $worktype_e_opt_ei
            })* else {
                $worktype_e_opt_e
            }))?
            $($worktype_e_other)?
        }
    }}
}

macro_rules! map_matches_simple {
    (
        $matches:ident,
        $work_string_i:literal => $worktype_i_path:path;
        $($work_string_ei:literal => $worktype_ei_path:path;)*
        _ => $worktype_e_path:path;
    ) => {{
        if $matches.is_present($work_string_i) {
            $worktype_i_path
        } $(else if $matches.is_present($work_string_ei) {
            $worktype_ei_path
        })* else {
            $worktype_e_path
        }
    }}
}

const GPU72_TYPES_AND_OPTS_HELP: &'static str = r"GPU to 72 work types and options:
    - Lucas-Lehmer trial factoring             --gpu72-lucas-lehmer-trial-factor
        - What makes sense                         --gpu72-what-makes-sense
        - Lowest trial factor level                --gpu72-lowest-trial-factor-level
        - Highest trial factor level               --gpu72-highest-trial-factor-level
        - Lowest exponent                          --gpu72-lowest-exponent
        - Oldest exponent                          --gpu72-oldest-exponent
        - Lone Mersenne Hunters bit-first          --gpu72-lone-mersenne-hunters-bit-first
        - Lone Mersenne Hunters depth-first        --gpu72-lone-mersenne-hunters-depth-first
        - Let GPU to 72 decide                     --gpu72-let-gpu72-decide
    - Double-check trial factoring             --gpu72-double-check-trial-factor
        - What makes sense                         --gpu72-what-makes-sense
        - Lowest trial factor level                --gpu72-lowest-trial-factor-level
        - Highest trial factor level               --gpu72-highest-trial-factor-level
        - Lowest exponent                          --gpu72-lowest-exponent
        - Oldest exponent                          --gpu72-oldest-exponent
        - Double-check already done                --gpu72-double-check-already-done
        - Let GPU to 72 decide                     --gpu72-let-gpu72-decide
    - Lucas-Lehmer P-1 factoring               --gpu72-lucas-lehmer-p1
        - What makes sense                         --gpu72-what-makes-sense
        - Lowest exponent                          --gpu72-lowest-exponent
        - Oldest exponent                          --gpu72-oldest-exponent";

pub fn request_from_args() -> Result<Options, String> {
    let current_dir = format!("{}", current_dir().unwrap().display());
    let matches = App::new("primenet-rs")
        .version("1.0.0")
        .about("Interface to request from and report to Primenet (GIMPS) and GPU to 72.")
        .author("Aurorans Solis")
        .subcommand(
            App::new("p95")
                .author("Aurorans Solis")
                .version("1.0.0")
                .about("Interface to request from and report to Primenet (GIMPS)")
                .arg(
                    Arg::with_name("work-directory")
                        .short('w')
                        .long("work-directory")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("WORKDIR")
                        .default_value(&current_dir)
                        .validator(directory_validator)
                        .help("Working directory with worktodo.txt/worktodo.ini and results.txt")
                )
                .arg(
                    Arg::with_name("num-cache")
                        .short('n')
                        .long("num-cache")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("NUM_CACHE")
                        .default_value("1")
                        .validator(numeric_validator)
                        .help("Number of assignments to cache")
                )
                .arg(
                    Arg::with_name("timeout")
                        .short('t')
                        .long("timeout")
                        .number_of_values(1)
                        .value_name("TIMEOUT")
                        .default_value("0")
                        .validator(numeric_validator)
                        .help(
                            "Seconds to wait between network updates. Use 0 for a single update \
                                without looping."
                        ),
                )
                .group(
                    ArgGroup::with_name("general options")
                        .args(&["work-directory", "num-cache", "timeout"])
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("username")
                        .long("p95-username")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("USERNAME")
                        .validator(p95_username_validator)
                        .help("Primenet username")
                        .required_unless("username-file")
                )
                .arg(
                    Arg::with_name("username-file")
                        .long("p95-username-file")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("FILE_PATH")
                        .validator(file_validator)
                        .help("Path to file containing Primenet username")
                        .required_unless("username")
                )
                .arg(
                    Arg::with_name("password")
                        .long("p95-password")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("PASSWORD")
                        .help("Primenet password")
                        .required_unless("password-file")
                )
                .arg(
                    Arg::with_name("password-file")
                        .long("p95-password-file")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("FILE_PATH")
                        .validator(file_validator)
                        .help("Path to file containing Primenet password")
                        .required_unless("password")
                )
                .arg(
                    Arg::with_name("trial-factoring")
                        .long("trial-factoring")
                        .visible_alias("tf")
                        .help("Request trial factoring work from Primenet")
                )
                .arg(
                    Arg::with_name("p1-factoring")
                        .long("p1-factoring")
                        .visible_alias("p1f")
                        .help("Request P-1 factoring work from Primenet")
                )
                .arg(
                    Arg::with_name("ecm-factoring")
                        .long("ecm-factoring")
                        .visible_alias("ecmf")
                        .help("Request Elliptic Curve Method factoring work from Primenet")
                )
                .arg(
                    Arg::with_name("ecm-factoring-of-mersenne-cofactors")
                        .long("ecm-factoring-of-mersenne-cofactors")
                        .visible_alias("ecmfomc")
                        .help(
                            "Request Elliptic Curve Method factoring of Mersenne cofactors work \
                            from Primenet"
                        )
                )
                .arg(
                    Arg::with_name("smallest-available-first-time-ll")
                        .long("smallest-available-first-time-ll")
                        .visible_alias("saftll")
                        .help("Request smallest available first-time Lucas-Lehmer work from Primenet")
                )
                .arg(
                    Arg::with_name("double-check-ll")
                        .long("double-check-ll")
                        .visible_alias("dcll")
                        .help("Request Lucas-Lehmer double-check work from Primenet")
                )
                .arg(
                    Arg::with_name("world-record-ll")
                        .long("world-record-ll")
                        .visible_alias("wrll")
                        .help("Request world record-sized Lucas-Lehmer tests from Primenet")
                )
                .arg(
                    Arg::with_name("100m-digits-ll")
                        .long("100m-digits-ll")
                        .visible_alias("100mdll")
                        .help("Request 100M digits Lucas-Lehmer tests from Primenet")
                )
                .arg(
                    Arg::with_name("smallest-available-first-time-prp")
                        .long("smallest-available-first-time-prp")
                        .visible_alias("saftprp")
                        .help("Request smallest available probable prime work from Primenet")
                )
                .arg(
                    Arg::with_name("double-check-prp")
                        .long("double-check-prp")
                        .visible_alias("dcprp")
                        .help("Request double-check of probable prime work from Primenet")
                )
                .arg(
                    Arg::with_name("world-record-prp")
                        .long("world-record-prp")
                        .visible_alias("wrprp")
                        .help("Request world record-sized probable prime work from Primenet")
                )
                .arg(
                    Arg::with_name("100m-digits-prp")
                        .long("100m-digits-prp")
                        .visible_alias("100mdprp")
                        .help("Request 100M digits probable prime tests from Primenet")
                )
                .arg(
                    Arg::with_name("first-prp-on-mersenne-cofactors")
                        .long("first-prp-on-mersenne-cofactors")
                        .visible_alias("fprpomc")
                        .help("Request first PRP tests on Mersenne cofactors from Primenet")
                )
                .arg(
                    Arg::with_name("double-check-prp-on-mersenne-cofactors")
                        .long("double-check-prp-on-mersenne-cofactors")
                        .visible_alias("dcprpomc")
                        .help(
                            "Request double-checks of PRP tests on Mersenne cofactors from Primenet"
                        )
                )
                .group(
                    ArgGroup::with_name("worktype")
                        .args(&[
                            "trial-factoring",
                            "p1-factoring",
                            "ecm-factoring",
                            "ecm-factoring-of-mersenne-cofactors",
                            "smallest-available-first-time-ll",
                            "double-check-ll",
                            "world-record-ll",
                            "100m-digits-ll",
                            "smallest-available-first-time-prp",
                            "double-check-prp",
                            "world-record-prp",
                            "100m-digits-prp",
                            "first-prp-on-mersenne-cofactors",
                            "double-check-prp-on-mersenne-cofactors",
                        ])
                        .required(true)
                        .multiple(false)
                )
        )
        .subcommand(
            App::new("gpu72")
                .author("Aurorans Solis")
                .version("1.0.0")
                .about("Interface to request from and report to GPU to 72")
                .after_help(GPU72_TYPES_AND_OPTS_HELP)
                .arg(
                    Arg::with_name("work-directory")
                        .short('w')
                        .long("work-directory")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("WORKDIR")
                        .default_value(&current_dir)
                        .validator(directory_validator)
                        .help("Working directory with worktodo.txt/worktodo.ini and results.txt")
                )
                .arg(
                    Arg::with_name("num-cache")
                        .short('n')
                        .long("num-cache")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("NUM_CACHE")
                        .default_value("1")
                        .validator(numeric_validator)
                        .help("Number of assignments to cache")
                )
                .arg(
                    Arg::with_name("timeout")
                        .short('t')
                        .long("timeout")
                        .number_of_values(1)
                        .value_name("TIMEOUT")
                        .default_value("0")
                        .validator(numeric_validator)
                        .help(
                            "Seconds to wait between network updates. Use 0 for a single update \
                                without looping."
                        )
                )
                .group(
                    ArgGroup::with_name("general options")
                        .args(&["work-directory", "num-cache", "timeout"])
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("gpu72-username")
                        .long("gpu72-username")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("USERNAME")
                        .validator(gpu72_username_validator)
                        .help("GPU to 72 username")
                        .required_unless("gpu72-username-file")
                        .conflicts_with("gpu72-username-file")
                )
                .arg(
                    Arg::with_name("gpu72-username-file")
                        .long("gpu72-username-file")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("FILE_PATH")
                        .validator(file_validator)
                        .help("Path to file containing GPU to 72 username")
                        .required_unless("gpu72-username")
                        .conflicts_with("gpu72-username")
                )
                .arg(
                    Arg::with_name("gpu72-password")
                        .long("gpu72-password")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("PASSWORD")
                        .help("GPU to 72 password")
                        .required_unless("gpu72-password-file")
                        .conflicts_with("gpu72-password-file")
                )
                .arg(
                    Arg::with_name("gpu72-password-file")
                        .long("gpu72-password-file")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("FILE_PATH")
                        .validator(file_validator)
                        .help("Path to file containing GPU to 72 password")
                        .required_unless("gpu72-password")
                        .conflicts_with("gpu72-password")
                )
                .arg(
                    Arg::with_name("p95-username")
                        .long("p95-username")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("USERNAME")
                        .validator(p95_username_validator)
                        .help("Primenet username")
                )
                .arg(
                    Arg::with_name("p95-username-file")
                        .long("p95-username-file")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("FILE_PATH")
                        .validator(file_validator)
                        .help("Path to file containing Primenet username")
                )
                .group(
                    ArgGroup::with_name("p95-user")
                        .args(&["p95-username", "p95-username-file"])
                        .multiple(false)
                )
                .arg(
                    Arg::with_name("p95-password")
                        .long("p95-password")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("PASSWORD")
                        .help("Primenet password")
                )
                .arg(
                    Arg::with_name("p95-password-file")
                        .long("p95-password-file")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("FILE_PATH")
                        .validator(file_validator)
                        .help("Path to file containing Primenet password")
                )
                .group(
                    ArgGroup::with_name("p95-pass")
                        .args(&["p95-password", "p95-password-file"])
                        .multiple(false)
                )
                .group(
                    ArgGroup::with_name("p95-credentials")
                        .args(&["p95-user", "p95-pass"])
                        .multiple(true)
                        .requires_all(&["p95-user", "p95-pass"])
                )
                .arg(
                    Arg::with_name("p95-fallback")
                        .long("p95-fallback")
                        .help(
                            "Fall back to Primenet if requests to GPU to 72 fail or it has no \
                            work. Always fetches trial factor work, regardless of GPU to 72 work \
                            type and options."
                        )
                        .requires_all(&["p95-credentials", "p95-fallback-type"])
                )
                .arg(
                    Arg::with_name("max-exponent")
                        .long("max-exponent")
                        .visible_alias("max-exp")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("NUM")
                        .validator(max_exp_validator)
                        .default_value("72")
                        .help("Upper limit of exponent")
                )
                .arg(
                    Arg::with_name("lucas-lehmer-trial-factor")
                        .visible_alias("lltf")
                        .long("lucas-lehmer-trial-factor")
                        .help("Request LL trial factoring work from GPU to 72")
                        .required_unless_one(&GPU72LLTF_LIST)
                        .conflicts_with_all(&GPU72LLTF_LIST)
                )
                .arg(
                    Arg::with_name("double-check-trial-factor")
                        .visible_alias("dctf")
                        .long("double-check-trial-factor")
                        .help("Request double-check trial factoring work from GPU to 72")
                        .required_unless_one(&GPU72DCTF_LIST)
                        .conflicts_with_all(&GPU72DCTF_LIST)
                )
                .arg(
                    Arg::with_name("lucas-lehmer-p1")
                        .visible_alias("llp1")
                        .long("lucas-lehmer-p1")
                        .help("Request LL P-1 work from GPU to 72")
                        .required_unless_one(&GPU72LLP1_LIST)
                        .conflicts_with_all(&GPU72LLP1_LIST)
                )
                .arg(
                    Arg::with_name("what-makes-most-sense")
                        .visible_alias("wmms")
                        .long("what-makes-most-sense")
                        .help("Ask GPU to 72 to assign whatever makes most sense.")
                        .required_unless_one(&GPU72WMS_LIST)
                        .conflicts_with_all(&GPU72WMS_LIST)
                )
                .arg(
                    Arg::with_name("lowest-trial-factor-level")
                        .visible_alias("ltfl")
                        .long("lowest-trial-factor-level")
                        .help("Request work of the lowest trial factoring level from GPU to 72")
                        .required_unless_one(&GPU72LTFL_LIST)
                        .conflicts_with_all(&GPU72LTFL_LIST)
                )
                .arg(
                    Arg::with_name("highest-trial-factor-level")
                        .visible_alias("htfl")
                        .long("highest-trial-factor-level")
                        .help("Request work of the highest trial factoring level from GPU to 72")
                        .required_unless_one(&GPU72HTFL_LIST)
                        .conflicts_with_all(&GPU72HTFL_LIST)
                )
                .arg(
                    Arg::with_name("lowest-exponent")
                        .visible_alias("le")
                        .long("lowest-exponent")
                        .help("Request the lowest exponent for the selected work type from GPU \
                            to 72")
                        .required_unless_one(&GPU72LE_LIST)
                        .conflicts_with_all(&GPU72LE_LIST)
                )
                .arg(
                    Arg::with_name("oldest-exponent")
                        .visible_alias("oe")
                        .long("oldest-exponent")
                        .help("Request the oldest exponent for the selected work type from GPU \
                            to 72")
                        .required_unless_one(&GPU72OE_LIST)
                        .conflicts_with_all(&GPU72OE_LIST)
                )
                .arg(
                    Arg::with_name("double-check-already-done")
                        .visible_alias("dcad")
                        .long("double-check-already-done")
                        .help(
                            "Request double-check trial factoring work where a double check has \
                            already been done from GPU to 72"
                        )
                        .required_unless_one(&GPU72DCAD_LIST)
                        .conflicts_with_all(&GPU72DCAD_LIST)
                )
                .arg(
                    Arg::with_name("lone-mersenne-hunters-bit-first")
                        .visible_alias("lmh-bf")
                        .long("lone-mersenne-hunters-bit-first")
                        .help("Request LMH bit-first work from GPU to 72")
                        .required_unless_one(&GPU72LMHBF_LIST)
                        .conflicts_with_all(&GPU72LMHBF_LIST)
                )
                .arg(
                    Arg::with_name("lone-mersenne-hunters-depth-first")
                        .visible_alias("lmh-df")
                        .long("lone-mersenne-hunters-depth-first")
                        .help("Request LMH depth-first work from GPU to 72")
                        .required_unless_one(&GPU72LMHDF_LIST)
                        .conflicts_with_all(&GPU72LMHDF_LIST)
                )
                .arg(
                    Arg::with_name("let-gpu72-decide")
                        .visible_alias("lgpu72d")
                        .long("let-gpu72-decide")
                        .help("Let GPU to 72 decide what type of work to do.")
                        .required_unless_one(&GPU72LGPU72D_LIST)
                        .conflicts_with_all(&GPU72LGPU72D_LIST)
                )
        ).try_get_matches().map_err(|e| format!("{}", e))?;
    if let Some(matches) = matches.subcommand_matches("gpu72") {
        let gpu72_credentials = if matches.is_present("gpu72-userpass") {
            (
                matches.value_of("gpu72-username").unwrap().to_string(),
                matches.value_of("gpu72-password").unwrap().to_string(),
            )
        } else {
            let username_path = matches.value_of("gpu72-username-file").unwrap();
            let mut username_file = BufReader::new(File::open(username_path).unwrap());
            let mut username = String::new();
            username_file
                .read_to_string(&mut username)
                .map_err(|e| format!("Error reading username file '{}': {}", username_path, e))?;
            let username = username.trim().to_string();
            let password_path = matches.value_of("gpu72-password-file").unwrap();
            let mut password_file = BufReader::new(File::open(password_path).unwrap());
            let mut password = String::new();
            password_file
                .read_to_string(&mut password)
                .map_err(|e| format!("Error reading password file '{}': {}", password_path, e))?;
            let password = password.trim().to_string();
            (username, password)
        };
        let primenet_credentials = if matches.is_present("p95-fallback") {
            if matches.is_present("p95-userpass") {
                Some((
                    matches.value_of("p95-username").unwrap().to_string(),
                    matches.value_of("p95-password").unwrap().to_string(),
                ))
            } else {
                let username_path = matches.value_of("p95-username-file").unwrap();
                let mut username_file = BufReader::new(File::open(username_path).unwrap());
                let mut username = String::new();
                username_file.read_to_string(&mut username).map_err(|e| {
                    format!("Error reading username file '{}': {}", username_path, e)
                })?;
                let username = username.trim().to_string();
                let password_path = matches.value_of("p95-password-file").unwrap();
                let mut password_file = BufReader::new(File::open(password_path).unwrap());
                let mut password = String::new();
                password_file.read_to_string(&mut password).map_err(|e| {
                    format!("Error reading password file '{}': {}", password_path, e)
                })?;
                let password = password.trim().to_string();
                Some((username, password))
            }
        } else {
            None
        };
        let max_exp = matches
            .value_of("max-exponent")
            .unwrap()
            .parse::<u8>()
            .unwrap();
        let work_directory = matches.value_of("work-directory").unwrap().to_string();
        let num_cache = matches
            .value_of("num-cache")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let timeout = matches
            .value_of("timeout")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let general_options = GeneralOptions {
            work_directory,
            num_cache,
            timeout,
        };
        let work_type = map_matches!(
            matches,
            "lucas-lehmer-trial-factor" => Gpu72WorkType::LucasLehmerTrialFactor {
                "what-makes-sense" -> Gpu72LLTFWorkOption::WhatMakesSense;
                "lowest-trial-factor-level" -> Gpu72LLTFWorkOption::LowestTrialFactorLevel;
                "highest-trial-factor-level" -> Gpu72LLTFWorkOption::HighestTrialFactorLevel;
                "lowest-exponent" -> Gpu72LLTFWorkOption::LowestExponent;
                "oldest-exponent" -> Gpu72LLTFWorkOption::OldestExponent;
                "lone-mersenne-hunters-bit-first" -> Gpu72LLTFWorkOption::LmhBitFirst;
                "lone-mersenne-hunters-depth-first" -> Gpu72LLTFWorkOption::LmhDepthFirst;
                _ -> Gpu72LLTFWorkOption::LetGpu72Decide;
            }
            "double-check-trial-factor" => Gpu72WorkType::DoubleCheckTrialFactor {
                "what-makes-sense" -> Gpu72DCTFWorkOption::WhatMakesSense;
                "lowest-trial-factor-level" -> Gpu72DCTFWorkOption::LowestTrialFactorLevel;
                "highest-trial-factor-level" -> Gpu72DCTFWorkOption::HighestTrialFactorLevel;
                "lowest-exponent" -> Gpu72DCTFWorkOption::LowestExponent;
                "oldest-exponent" -> Gpu72DCTFWorkOption::OldestExponent;
                "double-check-already-done" -> Gpu72DCTFWorkOption::DoubleCheckAlreadyDone;
                _ -> Gpu72DCTFWorkOption::LetGpu72Decide;
            }
            _ => Gpu72WorkType::LucasLehmerP1 {
                "lowest-exponent" -> Gpu72LLP1WorkOption::LowestExponent;
                "oldest-exponent" -> Gpu72LLP1WorkOption::OldestExponent;
                _ -> Gpu72LLP1WorkOption::WhatMakesSense;
            }
        );
        Ok(Options::Gpu72(Gpu72Options {
            primenet_credentials,
            gpu72_credentials,
            work_type,
            max_exp,
            general_options,
        }))
    } else if let Some(matches) = matches.subcommand_matches("p95") {
        let username = if matches.is_present("username") {
            matches.value_of("username").unwrap().to_string()
        } else {
            let username_path = matches.value_of("username-file").unwrap();
            let mut username_file = BufReader::new(File::open(username_path).unwrap());
            let mut username = String::new();
            username_file
                .read_to_string(&mut username)
                .map_err(|e| format!("Error reading username file '{}': {}", username_path, e))?;
            username.trim().to_string()
        };
        let password = if matches.is_present("password") {
            matches.value_of("password").unwrap().to_string()
        } else {
            let password_path = matches.value_of("password-file").unwrap();
            let mut password_file = BufReader::new(File::open(password_path).unwrap());
            let mut password = String::new();
            password_file
                .read_to_string(&mut password)
                .map_err(|e| format!("Error reading password file '{}': {}", password_path, e))?;
            password.trim().to_string()
        };
        let credentials = (username, password);
        let work_directory = matches.value_of("work-directory").unwrap().to_string();
        let num_cache = matches
            .value_of("num-cache")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let timeout = matches
            .value_of("timeout")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let general_options = GeneralOptions {
            work_directory,
            num_cache,
            timeout,
        };
        let work_type = map_matches_simple!(
            matches,
            "trial-factoring" => PrimenetWorkType::TrialFactoring;
            "p1-factoring" => PrimenetWorkType::P1Factoring;
            "ecm-factoring" => PrimenetWorkType::EcmFactoring;
            "ecm-factoring-of-mersenne-cofactors" =>
                PrimenetWorkType::EcmFactoringOfMersenneCofactors;
            "smallest-available-first-time-ll" =>
                PrimenetWorkType::SmallestAvailableFirstTimeLlTests;
            "double-check-ll" => PrimenetWorkType::DoubleCheckLlTests;
            "world-record-ll" => PrimenetWorkType::WorldRecordLlTests;
            "100m-digits-ll" => PrimenetWorkType::HundredMillionDigitsLlTests;
            "smallest-available-first-time-prp" =>
                PrimenetWorkType::SmallestAvailableFirstTimePrpTests;
            "double-check-prp" => PrimenetWorkType::DoubleCheckPrpTests;
            "world-record-prp" => PrimenetWorkType::WorldRecordPrpTests;
            "100m-digits-prp" => PrimenetWorkType::HundredMillionDigitsPrpTests;
            "first-prp-on-mersenne-cofactors" => PrimenetWorkType::FirstPrpTestsOnMersenneCofactors;
            _ => PrimenetWorkType::DoubleCheckPrpTestsOnMersenneCofactors;
        );
        Ok(Options::Primenet(PrimenetOptions {
            credentials,
            work_type,
            general_options,
        }))
    } else {
        Err("No subcommand specified.".to_string())
    }
}
