use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::js_sys;

// a newtype wrapper with string serialization and sensible arithmetic with Duration
// inner value is in nanoseconds since unix epoch
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        Self::from_millis(js_sys::Date::now() as u64)
    }

    pub fn from_nanos(nanos: u64) -> Self {
        Timestamp(nanos)
    }

    pub fn from_millis(millis: u64) -> Self {
        Timestamp(millis * 1_000_000)
    }

    pub fn to_nanos(&self) -> u64 {
        self.0
    }

    pub fn to_millis(&self) -> u64 {
        self.0 / 1_000_000
    }
}

impl std::ops::Add<std::time::Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: std::time::Duration) -> Self::Output {
        Timestamp(self.0 + rhs.as_nanos() as u64)
    }
}

impl std::ops::AddAssign<std::time::Duration> for Timestamp {
    fn add_assign(&mut self, rhs: std::time::Duration) {
        self.0 += rhs.as_nanos() as u64;
    }
}

impl std::ops::Sub<std::time::Duration> for Timestamp {
    type Output = Self;

    fn sub(self, rhs: std::time::Duration) -> Self::Output {
        Timestamp(self.0 - rhs.as_nanos() as u64)
    }
}

impl std::ops::SubAssign<std::time::Duration> for Timestamp {
    fn sub_assign(&mut self, rhs: std::time::Duration) {
        self.0 -= rhs.as_nanos() as u64;
    }
}

impl std::ops::Mul<std::time::Duration> for Timestamp {
    type Output = Self;

    fn mul(self, rhs: std::time::Duration) -> Self::Output {
        Timestamp(self.0 * rhs.as_nanos() as u64)
    }
}

impl std::ops::MulAssign<std::time::Duration> for Timestamp {
    fn mul_assign(&mut self, rhs: std::time::Duration) {
        self.0 *= rhs.as_nanos() as u64;
    }
}

impl std::ops::Div<std::time::Duration> for Timestamp {
    type Output = Self;

    fn div(self, rhs: std::time::Duration) -> Self::Output {
        Timestamp(self.0 / rhs.as_nanos() as u64)
    }
}

impl std::ops::DivAssign<std::time::Duration> for Timestamp {
    fn div_assign(&mut self, rhs: std::time::Duration) {
        self.0 /= rhs.as_nanos() as u64;
    }
}

impl From<js_sys::Date> for Timestamp {
    fn from(date: js_sys::Date) -> Self {
        Timestamp::from_millis(date.get_time() as u64)
    }
}

impl From<Timestamp> for js_sys::Date {
    fn from(ts: Timestamp) -> Self {
        js_sys::Date::new(&JsValue::from_f64(ts.to_millis() as f64))
    }
}

// serialize as string

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Timestamp(s.parse().unwrap()))
    }
}
