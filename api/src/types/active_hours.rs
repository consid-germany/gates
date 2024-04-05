#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcTime { hour: u8, minute: u8, second: u8}

impl UtcTime {
    fn _new(hour: u8, minute: u8, second: u8) -> Result<Self, &'static str> {
        if (0..=23).contains(&hour) && (0..=59).contains(&minute) && (0..=59).contains(&second) {
            Ok(Self { hour, minute, second })
        } else {
            Err("Invalid time format: must have 0 <= hour <= 23, 0 <= minute <= 59 and 0 <= second <= 59")
        }
    }

    fn _to_string(&self) -> String {
        format!("{:02}:{:02}:{:02}Z", self.hour, self.minute, self.second)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveHours { pub start: UtcTime, pub end: UtcTime }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveHoursPerWeek {
    pub monday: Option<ActiveHours>,
    pub tuesday: Option<ActiveHours>,
    pub wednesday: Option<ActiveHours>,
    pub thursday: Option<ActiveHours>,
    pub friday: Option<ActiveHours>,
    pub saturday: Option<ActiveHours>,
    pub sunday: Option<ActiveHours>,
}
