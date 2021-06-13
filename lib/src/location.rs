use std::fmt;
use std::ops::Range;

/// Range of offsets
pub type Position = Range<usize>;

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct PreciseLocation(pub Range<Location>);

impl PreciseLocation {
    pub fn new(start: Location, end: Location) -> PreciseLocation {
        PreciseLocation(start..end)
    }
}

impl Location {
    pub fn from(offset: usize, source: &str) -> Location {
        let mut total = 0;
        let mut line = 1;
        let mut column = 0;

        for ch in source.chars() {
            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }

            total += 1;

            if offset < total {
                break;
            }
        }

        Location { line, column }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}
