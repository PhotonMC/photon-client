#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockRotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

impl BlockRotation {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "none" => Some(Self::None),
            "clockwise_90" => Some(Self::Clockwise90),
            "180" => Some(Self::Clockwise180),
            "counterclockwise_90" => Some(Self::CounterClockwise90),
            _ => None,
        }
    }
}
