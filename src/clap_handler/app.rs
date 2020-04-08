use super::gpu72_work::*;
use super::p95_work::*;
use super::validators::*;
use clap::{App, Arg, ArgGroup, SubCommand};
use std::env::current_dir;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub struct GeneralOptions {
    work_directory: String,
    num_cache: usize,
    timeout: usize,
}

#[derive(Debug)]
pub struct PrimenetOptions {
    credentials: (String, String),
    work_type: PrimenetWorkType,
    general_options: GeneralOptions,
}

#[derive(Debug)]
pub struct Gpu72Options {
    primenet_credentials: Option<(String, String)>,
    gpu72_credentials: (String, String),
    fallback: bool,
    work_type: Gpu72WorkType,
    general_options: GeneralOptions,
}

#[derive(Debug)]
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

pub fn request_from_args() -> Result<Options, String> {
    let current_dir = format!("{}", current_dir().unwrap().display());
    let matches = App::new("primenet-rs")
        .version("1.0.0")
        .about("Interface to request from and report to Primenet (GIMPS) and GPU to 72.")
        .author("Aurorans Solis")
        .subcommand(
            SubCommand::with_name("p95")
                .author("Aurorans Solis")
                .version("1.0.0")
                .about("Interface to request from and report to Primenet (GIMPS)")
                .arg(
                    Arg::with_name("work-directory")
                        .short("w")
                        .long("work-directory")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("WORKDIR")
                        .default_value(&current_dir)
                        .validator(directory_validator)
                        .help("Working directory with worktodo.txt/worktodo.ini and results.txt"),
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
                        .help("Number of assignments to cache"),
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
                        ),
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
                        .visible_alias("p95-tf")
                        .long("p95-trial-factoring")
                        .help("Request trial factoring work from Primenet"),
                )
                .arg(
                    Arg::with_name("p95-p1-factoring")
                        .visible_alias("p95-p1")
                        .long("p95-p1-factoring")
                        .help("Request P-1 factoring work from Primenet"),
                )
                .arg(
                    Arg::with_name("p95-optimal-p1-factoring")
                        .visible_alias("p95-op1")
                        .long("p95-optimal-p1-factoring")
                        .help("Request optimal P-1 factoring work from Primenet"),
                )
                .arg(
                    Arg::with_name("p95-ecm-factoring")
                        .visible_alias("p95-ecm")
                        .long("p95-ecm-factoring")
                        .help("Request ECM factoring work from Primenet"),
                )
                .arg(
                    Arg::with_name("p95-lucas-lehmer-first-time")
                        .visible_alias("p95-llft")
                        .long("p95-lucas-lehmer-first-time")
                        .help("Request LL first time work from Primenet"),
                )
                .arg(
                    Arg::with_name("p95-lucas-lehmer-double-check")
                        .visible_alias("p95-lldc")
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
                        .visible_alias("p95-wmms")
                        .long("p95-what-makes-most-sense")
                        .help("Ask Primenet to assign whatever makes most sense"),
                )
                .arg(
                    Arg::with_name("p95-factoring-lmh")
                        .visible_alias("p95-flmh")
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
                        .visible_alias("p95-fts")
                        .long("p95-factoring-trial-sieve")
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
                        .visible_alias("p95-fp1s")
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
                        .visible_alias("p95-fp1l")
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
                        .visible_alias("p95-secm")
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
                        .visible_alias("p95-fecm")
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
                        .visible_alias("p95-cecm")
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
                        .visible_alias("p95-ll-ft")
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
                        .visible_alias("p95-ll-dc")
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
                        .visible_alias("p95-ll-wr")
                        .long("p95-lucas-lehmer-world-record")
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
                        .visible_alias("p95-ll-10md")
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
                        .visible_alias("p95-ll-100md")
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
                        .visible_alias("p95-ll-ntop1")
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
        )
        .subcommand(
            SubCommand::with_name("gpu72")
                .author("Aurorans Solis")
                .version("1.0.0")
                .about("Interface to request from and report to GPU to 72")
                .arg(
                    Arg::with_name("work-directory")
                        .short("w")
                        .long("work-directory")
                        .takes_value(true)
                        .number_of_values(1)
                        .value_name("WORKDIR")
                        .default_value(&current_dir)
                        .validator(directory_validator)
                        .help("Working directory with worktodo.txt/worktodo.ini and results.txt"),
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
                        .help("Number of assignments to cache"),
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
                        ),
                )
                .group(
                    ArgGroup::with_name("general options")
                        .args(&["work-directory", "num-cache", "timeout"])
                        .multiple(true),
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
                    Arg::with_name("gpu72-fallback")
                        .long("gpu72-fallback")
                        .help("Fall back to Primenet if requests to GPU to 72 fail or it has no work")
                        .requires("p95-credentials"),
                )
                .arg(
                    Arg::with_name("gpu72-lucas-lehmer-trial-factor")
                        .visible_alias("gpu72-lltf")
                        .long("gpu72-lucas-lehmer-trial-factor")
                        .help("Request LL trial factoring work from GPU to 72")
                )
                .arg(
                    Arg::with_name("gpu72-double-check-trial-factor")
                        .visible_alias("gpu72-dctf")
                        .long("gpu72-double-check-trial-factor")
                        .help("Request double-check trial factoring work from GPU to 72")
                )
                .arg(
                    Arg::with_name("gpu72-lucas-lehmer-p1")
                        .visible_alias("gpu72-llp1")
                        .long("gpu72-lucas-lehmer-p1")
                        .help("Request LL P-1 work from GPU to 72")
                )
                .arg(
                    Arg::with_name("gpu72-double-check-p1")
                        .visible_alias("gpu72-dcp1")
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
                        .visible_alias("gpu72-wmms")
                        .long("gpu72-what-makes-most-sense")
                        .help("Ask GPU to 72 to assign whatever makes most sense.")
                        .conflicts_with("gpu72-double-check-p1")
                )
                .arg(
                    Arg::with_name("gpu72-lowest-trial-factor-level")
                        .visible_alias("gpu72-ltfl")
                        .long("gpu72-lowest-trial-factor-level")
                        .help("Request work of the lowest trial factoring level from GPU to 72")
                        .conflicts_with_all(&["gpu72-lucas-lehmer-p1", "gpu72-double-check-p1"])
                )
                .arg(
                    Arg::with_name("gpu72-highest-trial-factor-level")
                        .visible_alias("gpu72-htfl")
                        .long("gpu72-highest-trial-factor-level")
                        .help("Request work of the highest trial factoring level from GPU to 72")
                        .conflicts_with_all(&["gpu72-lucas-lehmer-p1", "gpu72-double-check-p1"])
                )
                .arg(
                    Arg::with_name("gpu72-lowest-exponent")
                        .visible_alias("gpu72-le")
                        .long("gpu72-lowest-exponent")
                        .help("Request the lowest exponent for the selected work type from GPU to 72")
                        .conflicts_with("gpu72-double-check-p1")
                )
                .arg(
                    Arg::with_name("gpu72-oldest-exponent")
                        .visible_alias("gpu72-oe")
                        .long("gpu72-oldest-exponent")
                        .help("Request the oldest exponent for the selected work type from GPU to 72")
                        .conflicts_with("gpu72-double-check-p1")
                )
                .arg(
                    Arg::with_name("gpu72-double-check-already-done")
                        .visible_alias("gpu72-dcad")
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
                        .visible_alias("gpu72-lmh-bf")
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
                        .visible_alias("gpu72-lmh-df")
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
                        .visible_alias("gpu72-lgpu72d")
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
        ).get_matches_safe().map_err(|e| format!("{}", e))?;
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
            let password_path = matches.value_of("gpu72-password-file").unwrap();
            let mut password_file = BufReader::new(File::open(password_path).unwrap());
            let mut password = String::new();
            password_file
                .read_to_string(&mut password)
                .map_err(|e| format!("Error reading password file '{}': {}", password_path, e))?;
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
                username_file.read_to_string(&mut username).map_err(|e| {
                    format!("Error reading username file '{}': {}", username_path, e)
                })?;
                let password_path = matches.value_of("p95-password-file").unwrap();
                let mut password_file = BufReader::new(File::open(password_path).unwrap());
                let mut password = String::new();
                password_file.read_to_string(&mut password).map_err(|e| {
                    format!("Error reading password file '{}': {}", password_path, e)
                })?;
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
            "gpu72-lucas-lehmer-trial-factor" => Gpu72WorkType::LucasLehmerTrialFactor {
                "gpu72-what-makes-sense" -> Gpu72LLTFWorkOption::WhatMakesSense;
                "gpu72-lowest-trial-factor-level" -> Gpu72LLTFWorkOption::LowestTrialFactorLevel;
                "gpu72-highest-trial-factor-level" -> Gpu72LLTFWorkOption::HighestTrialFactorLevel;
                "gpu72-lowest-exponent" -> Gpu72LLTFWorkOption::LowestExponent;
                "gpu72-oldest-exponent" -> Gpu72LLTFWorkOption::OldestExponent;
                "gpu72-lone-mersenne-hunters-bit-first" -> Gpu72LLTFWorkOption::LmhBitFirst;
                "gpu72-lone-mersenne-hunters-depth-first" -> Gpu72LLTFWorkOption::LmhDepthFirst;
                _ -> Gpu72LLTFWorkOption::LetGpu72Decide;
            }
            "gpu72-double-check-trial-factor" => Gpu72WorkType::DoubleCheckTrialFactor {
                "gpu72-what-makes-sense" -> Gpu72DCTFWorkOption::WhatMakesSense;
                "gpu72-lowest-trial-factor-level" -> Gpu72DCTFWorkOption::LowestTrialFactorLevel;
                "gpu72-highest-trial-factor-level" -> Gpu72DCTFWorkOption::HighestTrialFactorLevel;
                "gpu72-lowest-exponent" -> Gpu72DCTFWorkOption::LowestExponent;
                "gpu72-oldest-exponent" -> Gpu72DCTFWorkOption::OldestExponent;
                "gpu72-double-check-already-done" -> Gpu72DCTFWorkOption::DoubleCheckAlreadyDone;
                _ -> Gpu72DCTFWorkOption::LetGpu72Decide;
            }
            "gpu72-lucas-lehmer-p1" => Gpu72WorkType::LucasLehmerP1 {
                "gpu72-lowest-exponent" -> Gpu72LLP1WorkOption::LowestExponent;
                "gpu72-oldest-exponent" -> Gpu72LLP1WorkOption::OldestExponent;
                _ -> Gpu72LLP1WorkOption::WhatMakesSense;
            }
            _ => Gpu72WorkType::DoubleCheckP1 {
                _ -> Gpu72WorkType::DoubleCheckP1(
                    matches.value_of("gpu72-double-check-p1")
                        .unwrap()
                        .parse::<f32>()
                        .unwrap()
                );
            }
        );
        Ok(Options::Gpu72(Gpu72Options {
            primenet_credentials,
            gpu72_credentials,
            fallback,
            work_type,
            general_options,
        }))
    } else if let Some(matches) = matches.subcommand_matches("p95") {
        let credentials = if matches.is_present("p95-userpass") {
            (
                matches.value_of("p95-username").unwrap().to_string(),
                matches.value_of("p95-password").unwrap().to_string(),
            )
        } else {
            let username_path = matches.value_of("p95-username-file").unwrap();
            let mut username_file = BufReader::new(File::open(username_path).unwrap());
            let mut username = String::new();
            username_file
                .read_to_string(&mut username)
                .map_err(|e| format!("Error reading username file '{}': {}", username_path, e))?;
            let password_path = matches.value_of("p95-password-file").unwrap();
            let mut password_file = BufReader::new(File::open(password_path).unwrap());
            let mut password = String::new();
            password_file
                .read_to_string(&mut password)
                .map_err(|e| format!("Error reading password file '{}': {}", password_path, e))?;
            (username, password)
        };
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
            "p95-trial-factoring" => PrimenetWorkType::TrialFactoring {
                "p95-what-makes-most-sense" -> PrimenetTFOption::WhatMakesMostSense;
                "p95-factoring-lmh" -> PrimenetTFOption::FactoringLmh;
                _ -> PrimenetTFOption::FactoringTrialSieve;
            }
            "p95-p1-factoring" => PrimenetWorkType::P1Factoring {
                "p95-what-makes-most-sense" -> PrimenetP1FOption::WhatMakesMostSense;
                _ -> PrimenetP1FOption::FactoringP1Small;
            }
            "p95-optimal-p1-factoring" => PrimenetWorkType::OptimalP1Factoring {
                "p95-what-makes-most-sense" -> PrimenetOP1FOption::WhatMakesMostSense;
                _ -> PrimenetOP1FOption::FactoringP1Large;
            }
            "p95-ecm-factoring" => PrimenetWorkType::EcmFactoring {
                "p95-what-makes-most-sense" -> PrimenetEFOption::WhatMakesMostSense;
                "p95-smallish-ecm" -> PrimenetEFOption::SmallishEcm;
                "p95-fermat-ecm" -> PrimenetEFOption::FermatEcm;
                _ -> PrimenetEFOption::CunninghamEcm;
            }
            "p95-lucas-lehmer-first-time" => PrimenetWorkType::LlFirstTimeTest {
                "p95-what-makes-most-sense" -> PrimenetLLFTTOption::WhatMakesMostSense;
                "p95-lucas-lehmer-first-time-test" -> PrimenetLLFTTOption::LlFirstTimeTest;
                "p95-lucas-lehmer-world-record" -> PrimenetLLFTTOption::LlWorldRecord;
                "p95-lucas-lehmer-10m-digit" -> PrimenetLLFTTOption::Ll10mDigit;
                "p95-lucas-lehmer-100m-digit" -> PrimenetLLFTTOption::Ll100mDigit;
                _ -> PrimenetLLFTTOption::LlFirstTimeNoTrialOrP1;
            }
            _ => PrimenetWorkType::LlDoubleCheck {
                "p95-what-makes-most-sense" -> PrimenetLLDCOption::WhatMakesMostSense;
                _ -> PrimenetLLDCOption::LlDoubleCheck;
            }
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

/*
Alright, so I've got some command line argument parsing stuff I'm working on, and I can't figure out
why it's doing what it's doing. I'm trying to get the behaviour shown in the attached image, so I
have work type arguments grouped into a `p95-worktypes` group and work options (that each conflict
with options they're not compatible with (see center column of second table to see which one each
option is compatible with)) grouped into a `p95-workopts` group. Each group only allows one of its
arguments, and each group is required. However, when I run a valid type-opt pair, I get something
like `error: The argument '--p95-trial-factoring' cannot be used with one or more of the other
specified arguments` (for `--p95-trial-factoring` and `--p95-what-makes-most-sense`), and if I use
an invalid pair I get something like `error: The argument '--p95-trial-factoring' cannot be used
with '--p95-factoring-p1-small'` (for `--p95-trial-factoring` and `--p95-factoring-p1-small`).
Furthermore, the `--work-directory` argument appears to ignore the user input. I had it print out
the path it was using for the work directory, and it fails on
*/