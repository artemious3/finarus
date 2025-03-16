use std::ops::{DerefMut, Deref, Rem, Div, Sub, Add, AddAssign, SubAssign};
use serde::{Serialize, Deserialize};
use std::fmt::Display;

#[derive(Clone, Copy, Serialize,Deserialize, PartialEq, PartialOrd)]
pub struct Money (pub i32);

impl Deref for Money {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Rem<i32> for Money {
    type Output = Money;
    fn rem(self, rhs: i32) -> Self::Output {
        Money(self.0 % rhs)
    }
}

impl Div<i32> for Money {
    type Output = Money;
    fn div(self, rhs: i32) -> Self::Output {
        Money(self.0 / rhs)
    }
}

impl Add<i32> for Money {
    type Output = Money;
    fn add(self, rhs: i32) -> Self::Output {
        Money(self.0 + rhs)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Sub<i32> for Money{
    type Output = Money;
    fn sub(self, rhs: i32) -> Self::Output {
        Money(self.0 - rhs)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:02}", *self/100, *self%100)
    }
}

pub mod user;
pub mod auth;
pub mod deposit;
pub mod bank;
pub mod transaction;
pub mod validate;
pub mod account;
pub mod credit;
pub mod time;
