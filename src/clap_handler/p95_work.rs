#[derive(Copy, Clone, Debug)]
pub enum PrimenetWorkType {
    TrialFactoring(PrimenetTFOption),
    P1Factoring(PrimenetP1FOption),
    OptimalP1Factoring(PrimenetOP1FOption),
    EcmFactoring(PrimenetEFOption),
    LlFirstTimeTest(PrimenetLLFTTOption),
    LlDoubleCheck(PrimenetLLDCOption),
}

impl PrimenetWorkType {
    pub fn value(self) -> usize {
        match self {
            PrimenetWorkType::TrialFactoring(opt) => opt as usize,
            PrimenetWorkType::P1Factoring(opt) => opt as usize,
            PrimenetWorkType::OptimalP1Factoring(opt) => opt as usize,
            PrimenetWorkType::EcmFactoring(opt) => opt as usize,
            PrimenetWorkType::LlFirstTimeTest(opt) => opt as usize,
            PrimenetWorkType::LlDoubleCheck(opt) => opt as usize,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
// WMMS = 0
// Factoring LMH = 1
// Factoring trial (sieve) = 2
pub enum PrimenetTFOption {
    WhatMakesMostSense = 0,
    FactoringLmh = 1,
    FactoringTrialSieve = 2,
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
// WMMS = 0
// Factoring P-1 small = 3
pub enum PrimenetP1FOption {
    WhatMakesMostSense = 0,
    FactoringP1Small = 3,
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
// WMMS = 0
// Factoring P-1 large = 4
pub enum PrimenetOP1FOption {
    WhatMakesMostSense = 0,
    FactoringP1Large = 4,
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
// WMMS = 0
// Factoring ECM smallish Mersenne = 5
// Factoring Fermat ECM = 6
// Factoring Cunningham ECM = 7
pub enum PrimenetEFOption {
    WhatMakesMostSense = 0,
    SmallishMecm = 5,
    FermatEcm = 6,
    CunninghamEcm = 7,
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
// WMMS = 0
// LL first-time test = 100
// LL test world-record = 102
// LL test 10M digits = 103
// LL test 100M digits = 104
// LL first-time test with no trial or P-1 factoring = 105
pub enum PrimenetLLFTTOption {
    WhatMakesMostSense = 0,
    LlFirstTimeTest = 100,
    LlWorldRecord = 102,
    Ll10mDigits = 103,
    Ll100mDigits = 104,
    LlFirstTimeNoTrialOrP1 = 105,
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
// WMMS = 0
// LL double-check = 102
pub enum PrimenetLLDCOption {
    WhatMakesMostSense = 0,
    LlDoubleCheck = 102,
}
