use anyhow::Result;
use clap::ValueEnum;
pub use strum::IntoEnumIterator;
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Color(pub u8);

impl TryFrom<&str> for Color {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        let col = value.parse::<u8>()?;
        Ok(Color(col))
    }
}

impl TryFrom<String> for Color {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum Move {
    Stay,
    ToHalf,
    ToOther,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, Display, EnumIter)]
pub enum Distance {
    Same,
    Near,
    Far,
}

#[derive(ValueEnum, Debug, Display, Clone, Copy, PartialEq, Eq, EnumString, EnumIter)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Scheduler {
    Centralized,
    FSYNC,
    SSYNC,
    ASYNC_LC_Strict,
    ASYNC_LC_Atomic,
    ASYNC_CM_Atomic,
    ASYNC_Move_Atomic,
    ASYNC_Move_Regular,
    ASYNC_Move_Safe,
    ASYNC,
    ASYNC_Regular,
    ASYNC_Safe,
}

impl Color {
    pub fn iter_ncols(ncols: u8) -> impl Iterator<Item = Color> + Clone {
        (0..ncols).map(Color)
    }
}
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Move {
    pub fn as_code(&self) -> &str {
        static STAY: &str = "S";
        static TO_HALF: &str = "H";
        static TO_OTHER: &str = "O";
        match *self {
            Move::Stay => STAY,
            Move::ToHalf => TO_HALF,
            Move::ToOther => TO_OTHER,
        }
    }
}
impl TryFrom<&str> for Move {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        match value.to_uppercase().as_str() {
            "S" | "STAY" => Ok(Self::Stay),
            "H" | "HALF" | "TO_HALF" | "TOHALF" => Ok(Self::ToHalf),
            "O" | "OTHER" | "TO_OTHER" | "TOOTHER" => Ok(Self::ToOther),
            s => Err(anyhow::Error::msg(format!(
                "String does not describe a move: '{}'",
                s
            ))),
        }
    }
}
impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Stay => write!(f, "STAY"),
            Move::ToHalf => write!(f, "TO_HALF"),
            Move::ToOther => write!(f, "TO_OTHER"),
        }
    }
}

impl Default for Move {
    fn default() -> Self {
        Move::Stay
    }
}

impl Default for Distance {
    fn default() -> Self {
        Distance::Same
    }
}

impl Distance {
    pub fn try_parse(code: &str) -> Result<Self> {
        match code {
            "s" => Ok(Distance::Same),
            "d" | "n" => Ok(Distance::Near),
            "f" => Ok(Distance::Far),
            _ => anyhow::bail!("code not recognized as distance: \"{code}\""),
        }
    }
}

impl PartialOrd for Scheduler {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match (self, other) {
            (a, b) if a == b => Some(Equal),
            // from bottom
            (Self::Centralized, Self::FSYNC) | (Self::FSYNC, Self::Centralized) => None,
            (Self::Centralized | Self::FSYNC, _) => Some(Less),
            (_, Self::Centralized) | (_, Self::FSYNC) => Some(Greater),
            (Self::SSYNC, _) => Some(Less),
            (_, Self::SSYNC) => Some(Greater),
            // from top
            (_, Self::ASYNC_Safe) => Some(Less),
            (Self::ASYNC_Safe, _) => Some(Greater),
            (_, Self::ASYNC_Regular) => Some(Less),
            (Self::ASYNC_Regular, _) => Some(Greater),
            (_, Self::ASYNC) => Some(Less),
            (Self::ASYNC, _) => Some(Greater),
            (Self::ASYNC_LC_Strict, Self::ASYNC_LC_Atomic) => Some(Less),
            (Self::ASYNC_LC_Atomic, Self::ASYNC_LC_Strict) => Some(Greater),
            (Self::ASYNC_Move_Atomic, Self::ASYNC_Move_Regular | Self::ASYNC_Move_Safe) => {
                Some(Less)
            }
            (Self::ASYNC_Move_Regular | Self::ASYNC_Move_Safe, Self::ASYNC_Move_Atomic) => {
                Some(Greater)
            }
            (Self::ASYNC_Move_Regular, Self::ASYNC_Move_Safe) => Some(Less),
            (Self::ASYNC_Move_Safe, Self::ASYNC_Move_Regular) => Some(Greater),
            _ => None,
        }
    }
}

impl Scheduler {
    pub fn as_promela(&self) -> String {
        self.to_string().to_uppercase()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MyError;
impl std::error::Error for MyError {}
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyError")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        let c0 = Color(0);
        let c1 = Color(1);
        let c2 = Color(2);
        assert_eq!(c0, Color(0));
        assert_ne!(c0, c1);
        assert!(c0 < c1);
        assert!(c2 > c0);
        assert_eq!(std::cmp::max(c0, c2), c2);
        assert_eq!(std::cmp::min(c0, c2), c0);

        let mut ncols_iter = Color::iter_ncols(4);
        for i in 0..4 {
            assert_eq!(ncols_iter.next(), Some(Color(i)));
        }
        assert_eq!(ncols_iter.next(), None);
    }

    #[test]
    fn test_move() {
        assert!(Move::Stay < Move::ToHalf);
        assert!(Move::ToOther > Move::ToHalf);
        assert_eq!(std::cmp::max(Move::Stay, Move::ToHalf), Move::ToHalf);
        assert_eq!(std::cmp::min(Move::Stay, Move::ToHalf), Move::Stay);
        let mut iter = Move::iter();
        assert_eq!(iter.next(), Some(Move::Stay));
        assert_eq!(iter.next(), Some(Move::ToHalf));
        assert_eq!(iter.next(), Some(Move::ToOther));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_distance() {
        assert!(Distance::Same < Distance::Far);
        assert!(Distance::Far > Distance::Near);
        assert_eq!(
            std::cmp::max(Distance::Same, Distance::Near),
            Distance::Near
        );
        assert_eq!(
            std::cmp::min(Distance::Same, Distance::Near),
            Distance::Same
        );
        let mut iter = Distance::iter();
        assert_eq!(iter.next(), Some(Distance::Same));
        assert_eq!(iter.next(), Some(Distance::Near));
        assert_eq!(iter.next(), Some(Distance::Far));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_scheduler_ordering_reverse() {
        use std::cmp::Ordering::*;

        for lhs in Scheduler::iter() {
            for rhs in Scheduler::iter() {
                match lhs.partial_cmp(&rhs) {
                    None => assert_eq!(rhs.partial_cmp(&lhs), None),
                    Some(Equal) => assert_eq!(rhs.partial_cmp(&lhs), Some(Equal)),
                    Some(Less) => assert_eq!(rhs.partial_cmp(&lhs), Some(Greater)),
                    Some(Greater) => assert_eq!(rhs.partial_cmp(&lhs), Some(Less)),
                }
            }
        }
    }

    #[test]
    fn test_scheduler_ordering_transitivity() {
        use std::cmp::Ordering::*;

        for lhs in Scheduler::iter() {
            for rhs in Scheduler::iter() {
                for via in Scheduler::iter() {
                    match lhs.partial_cmp(&via) {
                        outcome @ (Some(Less) | Some(Greater)) => {
                            if via.partial_cmp(&rhs) == outcome {
                                assert_eq!(lhs.partial_cmp(&rhs), outcome);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    #[test]
    fn test_scheduler_ordering_irreflexivity() {
        use std::cmp::Ordering::*;
        for sched in Scheduler::iter() {
            assert_eq!(sched.partial_cmp(&sched), Some(Equal));
        }
    }
}
