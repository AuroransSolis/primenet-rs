#[derive(Copy, Clone, Debug)]
pub enum PrimenetWorkType {
    TrialFactoring(PrimenetTFOption),
    P1Factoring(PrimenetP1FOption),
    OptimalP1Factoring(PrimenetOP1FOption),
    EcmFactoring(PrimenetEFOption),
    LlFirstTimeTest(PrimenetLLFTTOption),
    LlDoubleCheck(PrimenetLLDCOption),
}

#[derive(Copy, Clone, Debug)]
pub enum PrimenetTFOption {
    WhatMakesMostSense,
    FactoringLmh,
    FactoringTrialSieve,
}

#[derive(Copy, Clone, Debug)]
pub enum PrimenetP1FOption {
    WhatMakesMostSense,
    FactoringP1Small,
}

#[derive(Copy, Clone, Debug)]
pub enum PrimenetOP1FOption {
    WhatMakesMostSense,
    FactoringP1Large,
}

#[derive(Copy, Clone, Debug)]
pub enum PrimenetEFOption {
    WhatMakesMostSense,
    SmallishMecm,
    FermatEcm,
    CunninghamEcm,
}

#[derive(Copy, Clone, Debug)]
pub enum PrimenetLLFTTOption {
    WhatMakesMostSense,
    LlFirstTimeTest,
    LlWorldRecord,
    Ll10mDigits,
    Ll100mDigits,
    LlFirstTimeNoTrialOrP1,
}

#[derive(Copy, Clone, Debug)]
pub enum PrimenetLLDCOption {
    WhatMakesMostSense,
    LlDoubleCheck,
}
