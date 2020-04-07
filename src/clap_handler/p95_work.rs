#[derive(Copy, Clone)]
pub enum PrimenetWorkType {
    TrialFactoring(PrimenetTFOption),
    P1Factoring(PrimenetP1FOption),
    OptimalP1Factoring(PrimenetOP1FOption),
    EcmFactoring(PrimenetEFOption),
    LlFirstTimeTest(PrimenetLLFTTOption),
    LlDoubleCheck(PrimenetLLDCOption),
}

#[derive(Copy, Clone)]
pub enum PrimenetTFOption {
    WhatMakesMostSense,
    FactoringLmh,
    FactoringTrialSieve,
}

#[derive(Copy, Clone)]
pub enum PrimenetP1FOption {
    WhatMakesMostSense,
    FactoringP1Small,
}

#[derive(Copy, Clone)]
pub enum PrimenetOP1FOption {
    WhatMakesMostSense,
    FactoringP1Large,
}

#[derive(Copy, Clone)]
pub enum PrimenetEFOption {
    WhatMakesMostSense,
    SmallishEcm,
    FermatEcm,
    CunninghamEcm,
}

#[derive(Copy, Clone)]
pub enum PrimenetLLFTTOption {
    WhatMakesMostSense,
    LlFirstTimeTest,
    LlWorldRecord,
    Ll10mDigit,
    Ll100mDigit,
    LlFirstTimeNoTrialOrP1,
}

#[derive(Copy, Clone)]
pub enum PrimenetLLDCOption {
    WhatMakesMostSense,
    LlDoubleCheck,
}
