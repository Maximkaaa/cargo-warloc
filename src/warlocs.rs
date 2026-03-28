use serde::Serialize;
use std::ops::{Add, AddAssign};

#[derive(Debug, Default, Copy, Clone, Serialize)]
pub struct Warlocs {
    pub file_count: u64,
    pub main: Locs,
    pub tests: Locs,
    pub examples: Locs,
}

#[derive(Debug, Default, Copy, Clone, Serialize)]
pub struct Locs {
    pub whitespaces: u64,
    pub code: u64,
    pub docs: u64,
    pub comments: u64,
    pub time: LocTimeStats,
}

/// Time stats for the lines of a code of agiven file.
/// Values are in seconds since UNIX epoch.
#[derive(Debug, Copy, Clone, Serialize)]
pub struct LocTimeStats {
    pub oldest: i128,
    pub newest: i128,
    pub average: i128,
}

/// Simple Serde-friendly wrapper struct which provides complete picture of the data.
#[derive(Debug, Serialize)]
pub struct SerializableStats {
    pub file_count: u64,
    pub main: Locs,
    pub tests: Locs,
    pub examples: Locs,
    pub totals: Locs,
}

impl From<&Warlocs> for SerializableStats {
    fn from(w: &Warlocs) -> Self {
        SerializableStats {
            file_count: w.file_count,
            main: w.main,
            tests: w.tests,
            examples: w.examples,
            totals: w.main + w.tests + w.examples,
        }
    }
}

impl Warlocs {
    pub fn whitespaces(&self) -> u64 {
        self.main.whitespaces + self.tests.whitespaces + self.examples.whitespaces
    }

    pub fn code(&self) -> u64 {
        self.main.code + self.tests.code + self.examples.code
    }

    pub fn docs(&self) -> u64 {
        self.main.docs + self.tests.docs + self.examples.docs
    }

    pub fn comments(&self) -> u64 {
        self.main.comments + self.tests.comments + self.examples.comments
    }

    pub fn sum(&self) -> u64 {
        self.whitespaces() + self.code() + self.docs() + self.comments()
    }

    pub fn serializable_totals(&self) -> SerializableStats {
        SerializableStats::from(self)
    }
}

impl Locs {
    pub fn sum(&self) -> u64 {
        self.whitespaces + self.code + self.docs + self.comments
    }
}

impl LocTimeStats {
    /// Accounts for the given seconds since UNIX Epoch timestamp.
    pub fn account(&mut self, timestamp: i128) {
        if self.oldest > timestamp {
            self.oldest = timestamp;
        }
        if self.newest < timestamp {
            self.newest = timestamp;
        }
        if self.average == 0 {
            self.average = timestamp
        } else {
            self.average = (self.average + timestamp) / 2
        }
    }
}

impl Add<Warlocs> for Warlocs {
    type Output = Self;

    fn add(self, rhs: Warlocs) -> Self::Output {
        Self {
            file_count: self.file_count + rhs.file_count,
            main: self.main + rhs.main,
            tests: self.tests + rhs.tests,
            examples: self.examples + rhs.examples,
        }
    }
}

impl AddAssign<Warlocs> for Warlocs {
    fn add_assign(&mut self, rhs: Warlocs) {
        *self = *self + rhs;
    }
}

impl Add<Locs> for Locs {
    type Output = Self;

    fn add(self, rhs: Locs) -> Self::Output {
        Self {
            whitespaces: self.whitespaces + rhs.whitespaces,
            code: self.code + rhs.code,
            docs: self.docs + rhs.docs,
            comments: self.comments + rhs.comments,
            time: self.time + rhs.time,
        }
    }
}

impl Default for LocTimeStats {
    fn default() -> Self {
        Self {
            oldest: i128::MAX,
            newest: i128::MIN,
            average: 0i128,
        }
    }
}

impl Add for LocTimeStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            oldest: self.oldest.min(rhs.oldest),
            newest: self.newest.max(rhs.newest),
            average: match (self.average, rhs.average) {
                (0, 0) => 0,
                (0, nondefault) => nondefault,
                (nondefault, 0) => nondefault,
                (l, r) => (l + r) / 2,
            },
        }
    }
}

impl AddAssign<LocTimeStats> for LocTimeStats {
    fn add_assign(&mut self, rhs: LocTimeStats) {
        *self = *self + rhs;
    }
}
