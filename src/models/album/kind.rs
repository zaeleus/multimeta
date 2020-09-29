use std::fmt;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Single,
    Ep,
    Lp,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Kind::Single => write!(f, "single"),
            Kind::Ep => write!(f, "ep"),
            Kind::Lp => write!(f, "lp"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        assert_eq!(Kind::Single.to_string(), "single");
        assert_eq!(Kind::Ep.to_string(), "ep");
        assert_eq!(Kind::Lp.to_string(), "lp");
    }
}
