use serde::Serialize;
use std::{
    iter::Sum,
    ops::{Add, AddAssign},
};

#[derive(Debug, Default, Copy, Clone, Serialize)]
pub struct Warlocs {
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
}

impl Locs {
    pub fn sum(&self) -> u64 {
        self.whitespaces + self.code + self.docs + self.comments
    }
}

impl Sum for Warlocs {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Warlocs::default(), Warlocs::add)
    }
}

impl Add<Warlocs> for Warlocs {
    type Output = Self;

    fn add(self, rhs: Warlocs) -> Self::Output {
        Self {
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
        }
    }
}
