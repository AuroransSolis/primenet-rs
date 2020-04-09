#[derive(Copy, Clone, Debug)]
pub enum Gpu72WorkType {
    LucasLehmerTrialFactor(Gpu72LLTFWorkOption),
    DoubleCheckTrialFactor(Gpu72DCTFWorkOption),
    LucasLehmerP1(Gpu72LLP1WorkOption),
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
