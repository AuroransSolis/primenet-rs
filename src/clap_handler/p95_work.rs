#[derive(Copy, Clone)]
pub enum PrimenetWorkType {
    TrialFactoring(PrimenetTFOption),
    P1Factoring(PrimenetP1FOption),
    OptimalP1Factoring(PrimenetOP1FOption),
    EcmFactoring(PrimenetEFOption),
    LlFirstTimeTest(PrimenetLLFTTOption),
    LlDoubleCheck(PrimenetLLDCOption),
}

pub enum PrimenetTFOption {
    WhatMakesMostSense,
    FactoringLmh,
    FactoringTrialSieve,
}

pub enum PrimenetP1FOption {
    WhatMakesMostSense,
    FactoringP1Small,
}

pub enum PrimenetOP1FOption {
    WhatMakesMostSense,
    FactoringP1Large,
}

pub enum PrimenetEFOption {
    WhatMakesMostSense,
    SmallishEcm,
    FermatEcm,
    CunninghamEcm,
}

pub enum PrimenetLLFTTOption {
    WhatMakesMostSense,
    LlFirstTimeTest,
    LlWorldRecord,
    Ll10mDigit,
    Ll100mDigit,
    LlFirstTimeNoTrialOrP1,
}

pub enum PrimenetLLDCOption {
    WhatMakesMostSense,
    LlDoubleCheck,
}
