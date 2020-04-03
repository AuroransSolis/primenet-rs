use clap::{App, Arg, ArgMatches};

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum PrimenetWorkType {
    SmallestAvailable = 100,
    DoubleCheck = 101,
    WorldRecord = 102,
    HundredMDigit = 104,
    SmallestAvailablePRP = 150,
    DoubleCheckPRP = 151,
    WorldRecordPRP = 152,
    HundredMDigitPRP = 153,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Gpu72WorkType {
    LucasLehmerTrialFactor(Gpu72LLTFWorkOption),
    DoubleCheckTrialFactor(Gpu72DCTFWorkOption),
    LucasLehmerP1(Gpu72LLP1WorkOption),
    // Guessing on the data type for this one. It's not apparent to me whether this should be a f32,
    // f64, or String going off the webpage source.
    DoubleCheckP1(f32),
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Gpu72LLTFWorkOption {
    WhatMakesSense = 0,
    LowestTrialFactorLevel = 1,
    HighestTrialFactorLevel = 2,
    LowestExponent = 3,
    OldestExponent = 4,
    LoneMersenneHuntersBitFirst = 6,
    LoneMersenneHuntersDepthFirst = 7,
    LetGpu72Decide = 9,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Gpu72DCTFWorkOption {
    WhatMakesSense = 0,
    LowestTrialFactorLevel = 1,
    HighestTrialFactorLevel = 2,
    LowestExponent = 3,
    OldestExponent = 4,
    DoubleCheckAlreadyDone = 5,
    LetGpu72Decide = 9,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Gpu72LLP1WorkOption {
    WhatMakesSense = 0,
    LowestExponent = 3,
    OldestExponent = 4,
}

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
    fallback_to_mersenne: bool,
    general_options: GeneralOptions,
}

pub enum Options {
    Primenet(PrimenetOptions),
    Gpu72(Gpu72Options),
}

// todo: CLAP