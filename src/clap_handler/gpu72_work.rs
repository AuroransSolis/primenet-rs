#[derive(Copy, Clone, Debug)]
pub enum Gpu72WorkType {
    LucasLehmerTrialFactor(Gpu72LLTFWorkOption),
    DoubleCheckTrialFactor(Gpu72DCTFWorkOption),
    LucasLehmerP1(Gpu72LLP1WorkOption),
}

const LLTF_ADDR: &str = "https://www.gpu72.com/account/getassignments/lltf";
const DCTF_ADDR: &str = "https://www.gpu72.com/account/getassignments/dctf";
const LLP1_ADDR: &str = "https://www.gpu72.com/account/getassignments/llp-1";

impl Gpu72WorkType {
    pub fn as_str(&self) -> (&'static str, &'static str) {
        match self {
            Gpu72WorkType::LucasLehmerTrialFactor(opt) => (LLTF_ADDR, opt.as_str()),
            Gpu72WorkType::DoubleCheckTrialFactor(opt) => (DCTF_ADDR, opt.as_str()),
            Gpu72WorkType::LucasLehmerP1(opt) => (LLP1_ADDR, opt.as_str()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Gpu72LLTFWorkOption {
    WhatMakesSense,
    LowestTrialFactorLevel,
    HighestTrialFactorLevel,
    LowestExponent,
    OldestExponent,
    LmhBitFirst,
    LmhDepthFirst,
    LetGpu72Decide,
}

impl Gpu72LLTFWorkOption {
    fn as_str(&self) -> &'static str {
        match self {
            Gpu72LLTFWorkOption::WhatMakesSense => "0",
            Gpu72LLTFWorkOption::LowestTrialFactorLevel => "1",
            Gpu72LLTFWorkOption::HighestTrialFactorLevel => "2",
            Gpu72LLTFWorkOption::LowestExponent => "3",
            Gpu72LLTFWorkOption::OldestExponent => "4",
            Gpu72LLTFWorkOption::LmhBitFirst => "6",
            Gpu72LLTFWorkOption::LmhDepthFirst => "7",
            Gpu72LLTFWorkOption::LetGpu72Decide => "9",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Gpu72DCTFWorkOption {
    WhatMakesSense,
    LowestTrialFactorLevel,
    HighestTrialFactorLevel,
    LowestExponent,
    OldestExponent,
    DoubleCheckAlreadyDone,
    LetGpu72Decide,
}

impl Gpu72DCTFWorkOption {
    fn as_str(&self) -> &'static str {
        match self {
            Gpu72DCTFWorkOption::WhatMakesSense => "0",
            Gpu72DCTFWorkOption::LowestTrialFactorLevel => "1",
            Gpu72DCTFWorkOption::HighestTrialFactorLevel => "2",
            Gpu72DCTFWorkOption::LowestExponent => "3",
            Gpu72DCTFWorkOption::OldestExponent => "4",
            Gpu72DCTFWorkOption::DoubleCheckAlreadyDone => "5",
            Gpu72DCTFWorkOption::LetGpu72Decide => "9",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Gpu72LLP1WorkOption {
    WhatMakesSense,
    LowestExponent,
    OldestExponent,
}

impl Gpu72LLP1WorkOption {
    fn as_str(&self) -> &'static str {
        match self {
            Gpu72LLP1WorkOption::WhatMakesSense => "0",
            Gpu72LLP1WorkOption::LowestExponent => "3",
            Gpu72LLP1WorkOption::OldestExponent => "4",
        }
    }
}
