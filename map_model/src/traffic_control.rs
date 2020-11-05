use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum TrafficBehavior {
    RED,
    ORANGE,
    GREEN,
    STOP,
}

impl TrafficBehavior {
    pub fn is_red(self) -> bool {
        matches!(self, TrafficBehavior::RED)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TrafficLightSchedule {
    period: usize,
    green: usize,
    orange: usize,
    red: usize,
    offset: usize,
}

impl TrafficLightSchedule {
    pub fn from_basic(green: usize, orange: usize, red: usize, offset: usize) -> Self {
        Self {
            period: green + orange + red,
            green,
            orange,
            red,
            offset,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TrafficControl {
    Always,
    Light(TrafficLightSchedule),
    StopSign,
}

impl TrafficControl {
    pub fn is_always(&self) -> bool {
        matches!(self, TrafficControl::Always)
    }

    pub fn is_stop_sign(&self) -> bool {
        matches!(self, TrafficControl::StopSign)
    }

    pub fn is_light(&self) -> bool {
        matches!(self, TrafficControl::Light(_))
    }

    pub fn get_behavior(&self, seconds: u32) -> TrafficBehavior {
        match self {
            TrafficControl::Always => TrafficBehavior::GREEN,
            TrafficControl::Light(schedule) => {
                let remainder = (seconds as usize + schedule.offset) % schedule.period;
                if remainder < schedule.green {
                    TrafficBehavior::GREEN
                } else if remainder < schedule.green + schedule.orange {
                    TrafficBehavior::ORANGE
                } else {
                    TrafficBehavior::RED
                }
            }
            TrafficControl::StopSign => TrafficBehavior::STOP,
        }
    }
}
