#[derive(Copy, Clone, Debug)]
pub enum PrimenetWorkType {
    TrialFactoring,
    P1Factoring,
    EcmFactoring,
    EcmFactoringOfMersenneCofactors,
    SmallestAvailableFirstTimeLlTests,
    DoubleCheckLlTests,
    WorldRecordLlTests,
    HundredMillionDigitsLlTests,
    SmallestAvailableFirstTimePrpTests,
    DoubleCheckPrpTests,
    WorldRecordPrpTests,
    HundredMillionDigitsPrpTests,
    FirstPrpTestsOnMersenneCofactors,
    DoubleCheckPrpTestsOnMersenneCofactors,
}

impl PrimenetWorkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimenetWorkType::TrialFactoring => "2",
            PrimenetWorkType::P1Factoring => "4",
            PrimenetWorkType::EcmFactoring => "5",
            PrimenetWorkType::EcmFactoringOfMersenneCofactors => "8",
            PrimenetWorkType::SmallestAvailableFirstTimeLlTests => "100",
            PrimenetWorkType::DoubleCheckLlTests => "101",
            PrimenetWorkType::WorldRecordLlTests => "102",
            PrimenetWorkType::HundredMillionDigitsLlTests => "104",
            PrimenetWorkType::SmallestAvailableFirstTimePrpTests => "150",
            PrimenetWorkType::DoubleCheckPrpTests => "151",
            PrimenetWorkType::WorldRecordPrpTests => "152",
            PrimenetWorkType::HundredMillionDigitsPrpTests => "153",
            PrimenetWorkType::FirstPrpTestsOnMersenneCofactors => "160",
            PrimenetWorkType::DoubleCheckPrpTestsOnMersenneCofactors => "161",
        }
    }
}
