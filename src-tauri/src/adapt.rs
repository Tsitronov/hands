use crate::model::HintLevel;

pub const CLEAN_LAPS_TO_ADVANCE: u32 = 2;
pub const MAX_ERRORS_FOR_CLEAN_LAP: u32 = 0;

pub struct LapOutcome {
    pub level: HintLevel,
    pub clean_streak: u32,
    pub advanced: bool,
}

pub fn after_lap(level: HintLevel, clean_streak: u32, errors: u32) -> LapOutcome {
    if errors > MAX_ERRORS_FOR_CLEAN_LAP {
        return LapOutcome { level, clean_streak: 0, advanced: false };
    }
    let streak = clean_streak + 1;
    if streak >= CLEAN_LAPS_TO_ADVANCE {
        let next = level.less_help();
        LapOutcome { level: next, clean_streak: 0, advanced: next != level }
    } else {
        LapOutcome { level, clean_streak: streak, advanced: false }
    }
}
