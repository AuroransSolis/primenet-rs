#[derive(Copy, Clone, Debug)]
pub enum Gpu72WorkType {
    LucasLehmerTrialFactor(Gpu72LLTFWorkOption),
    DoubleCheckTrialFactor(Gpu72DCTFWorkOption),
    LucasLehmerP1(Gpu72LLP1WorkOption),
    // Guessing on the data type for this one. It's not apparent to me whether this should be a f32,
    // f64, or String going off the webpage source.
    DoubleCheckP1(f32),
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

#[derive(Copy, Clone, Debug)]
pub enum Gpu72LLP1WorkOption {
    WhatMakesSense,
    LowestExponent,
    OldestExponent,
}
