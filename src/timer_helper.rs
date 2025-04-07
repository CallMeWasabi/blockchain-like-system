use std::sync::Arc;

use chrono::{TimeZone, Utc};
use mockall::automock;

pub type IntoTimerHelperShared = Arc<dyn IntoTimerHelper + Send + Sync>;

#[automock]
pub trait IntoTimerHelper {
    fn now(&self) -> i64;
}

pub enum TimerHelper {
    Directly,
    Mock,
}

impl TimerHelper {
    pub fn creation(&self) -> IntoTimerHelperShared {
        match self {
            Self::Directly => Arc::new(Self::Directly),
            Self::Mock => Arc::new(Self::Mock),
        }
    }
}

impl IntoTimerHelper for TimerHelper {
    fn now(&self) -> i64 {
        match self {
            Self::Directly => Utc::now().timestamp(),
            Self::Mock => Utc
                .with_ymd_and_hms(1970, 1, 1, 0, 0, 0)
                .unwrap()
                .timestamp(),
        }
    }
}
