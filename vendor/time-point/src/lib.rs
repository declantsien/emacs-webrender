use std::{
    mem,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

const NANOS_PER_MICRO: i64 = 1_000;
const NANOS_PER_MILLI: i64 = NANOS_PER_MICRO * 1_000;
const NANOS_PER_SEC: i64 = NANOS_PER_MILLI * 1_000;

fn zero_std_instant() -> std::time::Instant {
    if cfg!(unix) || cfg!(windows) {
        // https://github.com/rust-lang/rust/blob/master/src/libstd/sys/unix/time.rs
        // https://github.com/rust-lang/rust/blob/master/src/libstd/sys/windows/time.rs
        unsafe { mem::zeroed() }
    } else {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Duration {
    pub nanos: i64,
}

impl Duration {
    pub const fn new(nanos: i64) -> Self {
        Self { nanos }
    }

    pub const fn zero() -> Self {
        Self::new(0)
    }

    pub const fn from_secs(secs: i64) -> Self {
        Self::new(secs * NANOS_PER_SEC)
    }

    pub const fn from_millis(millis: i64) -> Self {
        Self::new(millis * NANOS_PER_MILLI)
    }

    pub const fn from_micros(micros: i64) -> Self {
        Self::new(micros * NANOS_PER_MICRO)
    }

    pub fn from_secs_f32(secs: f32) -> Self {
        Self::new((secs * NANOS_PER_SEC as f32).round() as i64)
    }

    pub fn from_secs_f64(secs: f64) -> Self {
        Self::new((secs * NANOS_PER_SEC as f64).round() as i64)
    }

    pub fn as_secs_f32(self) -> f32 {
        self.nanos as f32 / NANOS_PER_SEC as f32
    }

    pub fn as_secs_f64(self) -> f64 {
        self.nanos as f64 / NANOS_PER_SEC as f64
    }

    pub fn div_duration_f32(self, rhs: Self) -> f32 {
        self.as_secs_f32() / rhs.as_secs_f32()
    }

    pub fn div_duration_f64(self, rhs: Self) -> f64 {
        self.as_secs_f64() / rhs.as_secs_f64()
    }

    pub fn clamp(self, min: Duration, max: Duration) -> Self {
        assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    pub fn from_std_duration(std: std::time::Duration) -> Self {
        let result = std.as_nanos();
        debug_assert!(result <= i64::max_value() as u128);
        Self::new(result as _)
    }

    pub fn into_std_duration(self) -> std::time::Duration {
        debug_assert!(self.nanos >= 0);
        std::time::Duration::from_nanos(self.nanos as _)
    }
}

impl From<std::time::Duration> for Duration {
    fn from(std: std::time::Duration) -> Self {
        Self::from_std_duration(std)
    }
}

impl Into<std::time::Duration> for Duration {
    fn into(self) -> std::time::Duration {
        self.into_std_duration()
    }
}

impl Add for Duration {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.nanos + rhs.nanos)
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs
    }
}

impl Sub for Duration {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.nanos - rhs.nanos)
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs
    }
}

impl Neg for Duration {
    type Output = Self;
    fn neg(self) -> Self {
        Duration::zero() - self
    }
}

impl Mul<i32> for Duration {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        Self::new(self.nanos * rhs as i64)
    }
}

impl Mul<Duration> for i32 {
    type Output = Duration;
    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

impl MulAssign<i32> for Duration {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs
    }
}

impl Mul<f32> for Duration {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::from_secs_f32(self.as_secs_f32() * rhs)
    }
}

impl Mul<Duration> for f32 {
    type Output = Duration;
    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

impl MulAssign<f32> for Duration {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs
    }
}

impl Mul<f64> for Duration {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::from_secs_f64(self.as_secs_f64() * rhs)
    }
}

impl Mul<Duration> for f64 {
    type Output = Duration;
    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

impl MulAssign<f64> for Duration {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs
    }
}

impl Div<i32> for Duration {
    type Output = Self;
    fn div(self, rhs: i32) -> Self {
        Self::new(self.nanos / rhs as i64)
    }
}

impl DivAssign<i32> for Duration {
    fn div_assign(&mut self, rhs: i32) {
        *self = *self / rhs
    }
}

impl Div<f32> for Duration {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::from_secs_f32(self.as_secs_f32() / rhs)
    }
}

impl DivAssign<f32> for Duration {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs
    }
}

impl Div<f64> for Duration {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self::from_secs_f64(self.as_secs_f64() / rhs)
    }
}

impl DivAssign<f64> for Duration {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs
    }
}

impl Rem for Duration {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        Self::new(self.nanos % rhs.nanos)
    }
}

impl RemAssign for Duration {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct TimePoint {
    pub nanos_since_zero: i64,
}

impl TimePoint {
    /// This does not specifically require using unix time, however, that's probably expected.
    pub const fn new(nanos_since_zero: i64) -> Self {
        Self { nanos_since_zero }
    }

    pub const fn zero() -> Self {
        Self::new(0)
    }

    pub fn from_secs_f32(secs_since_zero: f32) -> Self {
        Self::zero() + Duration::from_secs_f32(secs_since_zero)
    }

    pub fn from_secs_f64(secs_since_zero: f64) -> Self {
        Self::zero() + Duration::from_secs_f64(secs_since_zero)
    }

    pub fn as_secs_f32(self) -> f32 {
        self.duration_from_zero().as_secs_f32()
    }

    pub fn as_secs_f64(self) -> f64 {
        self.duration_from_zero().as_secs_f64()
    }

    pub fn duration_from_zero(self) -> Duration {
        self - Self::zero()
    }

    pub fn from_std_instant(rhs: std::time::Instant) -> Self {
        Self::new(Duration::from_std_duration(rhs - zero_std_instant()).nanos)
    }

    pub fn into_std_instant(self) -> std::time::Instant {
        zero_std_instant() + self.duration_from_zero().into_std_duration()
    }
}

impl From<std::time::Instant> for TimePoint {
    fn from(std: std::time::Instant) -> Self {
        Self::from_std_instant(std)
    }
}

impl Into<std::time::Instant> for TimePoint {
    fn into(self) -> std::time::Instant {
        self.into_std_instant()
    }
}

impl Add<Duration> for TimePoint {
    type Output = TimePoint;
    fn add(self, rhs: Duration) -> Self {
        Self::new(self.nanos_since_zero + rhs.nanos)
    }
}

impl AddAssign<Duration> for TimePoint {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs
    }
}

impl Add<TimePoint> for Duration {
    type Output = TimePoint;
    fn add(self, rhs: TimePoint) -> TimePoint {
        rhs + self
    }
}

impl Sub<Duration> for TimePoint {
    type Output = TimePoint;
    fn sub(self, rhs: Duration) -> Self {
        self + -rhs
    }
}

impl SubAssign<Duration> for TimePoint {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output = Duration;
    fn sub(self, rhs: TimePoint) -> Duration {
        Duration::new(self.nanos_since_zero - rhs.nanos_since_zero)
    }
}

#[test]
fn std_compat() {
    let now = std::time::Instant::now();
    let now2 = TimePoint::from_std_instant(now).into_std_instant();
    assert_eq!(now, now2);

    let duration = std::time::Duration::from_millis(1_000_042);
    let duration2 = Duration::from_std_duration(duration).into_std_duration();
    assert_eq!(duration, duration2);

    let std_now = std::time::Instant::now();
    let std_now_plus = std_now + std::time::Duration::from_millis(1_000_042);
    let now = TimePoint::from_std_instant(std_now);
    let now_plus = now + Duration::from_millis(1_000_042);
    assert_eq!(std_now_plus, now_plus.into_std_instant());
    assert_eq!(TimePoint::from_std_instant(std_now_plus), now_plus);
}
