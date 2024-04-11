use chrono::{DateTime, Utc};

#[cfg_attr(test, mockall::automock)]
pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

pub fn default() -> impl Clock {
    DefaultClock {}
}

struct DefaultClock;

impl Clock for DefaultClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
