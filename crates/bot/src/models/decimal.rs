use std::{
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

#[derive(Clone, Copy)]
pub struct Decimal {
    sign: i8,
    raw: u64,
}

impl Decimal {
    pub const DECIMAL: usize = 6;
    pub const SCALE: u64 = 10u64.pow(Self::DECIMAL as u32);
    pub const ZERO: Self = Self { sign: 1, raw: 0 };
    pub const MAX: u64 = u64::MAX / Self::SCALE;

    pub fn from_str_unchecked(value: &str) -> Self {
        Decimal::from_str(value).unwrap()
    }

    pub fn from_f64_unchecked(value: f64) -> Self {
        Decimal::try_from(value).unwrap()
    }

    pub fn is_zero(&self) -> bool {
        self.raw == 0
    }

    pub fn is_positive(&self) -> bool {
        self.sign > 0
    }

    pub fn is_negative(&self) -> bool {
        self.sign < 0
    }
}

impl FromStr for Decimal {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::try_from(s.to_string())
    }
}

impl TryFrom<f64> for Decimal {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_nan() || value.is_infinite() {
            return Err("Cannot convert NaN or infinite value to Decimal");
        }

        if value == 0.0 {
            return Ok(Self::ZERO);
        }

        if value.abs() > Self::MAX as f64 {
            return Err("Value exceeds Decimal maximum limit");
        }

        let sign = if value < 0.0 { -1 } else { 1 };
        let abs_value = value.abs();
        let scaled = abs_value * (Decimal::SCALE as f64);
        let raw = scaled.round() as u64;

        Ok(Self { sign, raw })
    }
}

impl From<u64> for Decimal {
    fn from(value: u64) -> Self {
        Self {
            sign: 1,
            raw: value * Decimal::SCALE,
        }
    }
}

impl TryFrom<String> for Decimal {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err("Empty string cannot be converted to Decimal");
        }

        let sign = if trimmed.starts_with('-') { -1 } else { 1 };
        let unsigned = if sign == -1 { &trimmed[1..] } else { trimmed };

        let parts: Vec<&str> = unsigned.split('.').collect();

        if parts.len() > 2 {
            return Err("Invalid Decimal format");
        }

        let integer_part = parts[0]
            .parse::<u64>()
            .map_err(|_| "Invalid integer part")?;

        let fractional_part = if parts.len() == 2 {
            let fraction_str = format!("{:0<width$}", parts[1], width = Decimal::DECIMAL);
            fraction_str[..Decimal::DECIMAL]
                .parse::<u64>()
                .map_err(|_| "Invalid fractional part")?
        } else {
            0
        };

        let raw = integer_part
            .checked_mul(Decimal::SCALE)
            .and_then(|v| v.checked_add(fractional_part))
            .ok_or("Decimal overflow")?;

        Ok(Self { sign, raw })
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.sign == other.sign && self.raw == other.raw
    }
}

impl Eq for Decimal {}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.sign != other.sign {
            return self.sign.cmp(&other.sign);
        }
        self.raw.cmp(&other.raw)
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.sign == other.sign {
            Self {
                sign: self.sign,
                raw: self.raw + other.raw,
            }
        } else {
            match self.raw.cmp(&other.raw) {
                std::cmp::Ordering::Greater => Self {
                    sign: self.sign,
                    raw: self.raw - other.raw,
                },
                std::cmp::Ordering::Less => Self {
                    sign: -self.sign,
                    raw: other.raw - self.raw,
                },
                std::cmp::Ordering::Equal => Self::ZERO,
            }
        }
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if self.sign != other.sign {
            Self {
                sign: self.sign,
                raw: self.raw + other.raw,
            }
        } else {
            match self.raw.cmp(&other.raw) {
                std::cmp::Ordering::Greater => Self {
                    sign: self.sign,
                    raw: self.raw - other.raw,
                },
                std::cmp::Ordering::Less => Self {
                    sign: -self.sign,
                    raw: other.raw - self.raw,
                },
                std::cmp::Ordering::Equal => Self::ZERO,
            }
        }
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.raw == 0 {
            panic!("Division by zero in Decimal division");
        }

        if self.raw == 0 {
            return Self::ZERO;
        }

        let sign = self.sign * other.sign;
        let raw = (self.raw as u128 * Decimal::SCALE as u128) / (other.raw as u128);
        Self {
            sign,
            raw: raw as u64,
        }
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        if self.raw == 0 || other.raw == 0 {
            return Self::ZERO;
        }

        let sign = self.sign * other.sign;
        let raw = (self.raw as u128 * other.raw as u128) / Decimal::SCALE as u128;

        if raw > Decimal::MAX as u128 {
            panic!("Decimal multiplication overflow");
        }

        Self {
            sign,
            raw: raw as u64,
        }
    }
}

impl AddAssign for Decimal {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Decimal {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl MulAssign for Decimal {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign_str = if self.sign < 0 { "-" } else { "" };
        let raw_str = format!("{:0>width$}", self.raw, width = Self::DECIMAL + 1);
        write!(
            f,
            "{}{}.{}",
            sign_str,
            &raw_str[..raw_str.len() - Self::DECIMAL],
            &raw_str[raw_str.len() - Self::DECIMAL..]
        )
    }
}

impl std::fmt::Debug for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Decimal({})", self)
    }
}

impl Hash for Decimal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sign.hash(state);
        self.raw.hash(state);
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod decimal_tests {
    use super::*;

    #[test]
    fn test_decimal_creation() {
        let d1 = Decimal::try_from(123.456789).unwrap();
        assert_eq!(d1.to_string(), "123.456789");

        let d2 = Decimal::try_from(-123.456789).unwrap();
        assert_eq!(d2.to_string(), "-123.456789");

        let d3 = Decimal::try_from(0u64).unwrap();
        assert_eq!(d3.to_string(), "0.000000");

        let d4 = Decimal::from(100u64);
        assert_eq!(d4.to_string(), "100.000000");

        let d5 = Decimal::try_from("123.456789".to_string()).unwrap();
        assert_eq!(d5.to_string(), "123.456789");

        let d6 = Decimal::try_from("-123.456789".to_string()).unwrap();
        assert_eq!(d6.to_string(), "-123.456789");

        let d7 = Decimal::try_from("0".to_string()).unwrap();
        assert_eq!(d7.to_string(), "0.000000");

        let d8 = Decimal::try_from("100.1231234112312456".to_string()).unwrap();
        assert_eq!(d8.to_string(), "100.123123");

        let d9 = Decimal::try_from("-100.1231234112312456".to_string()).unwrap();
        assert_eq!(d9.to_string(), "-100.123123");

        let d10 = Decimal::try_from("10012512312312312312.123123".to_string());
        assert!(d10.is_err(), "Should fail for too large value");
    }

    #[test]
    fn test_decimal_addition() {
        let mut d1 = Decimal::try_from(100).unwrap();
        let d2 = Decimal::try_from(50.0).unwrap();
        d1 += d2;
        assert_eq!(d1.to_string(), "150.000000");

        let d3 = Decimal::try_from(-30.0).unwrap();
        d1 += d3;
        assert_eq!(d1.to_string(), "120.000000");

        let mut d4 = Decimal::try_from(-120.0).unwrap();
        let d5 = Decimal::try_from(-20.0).unwrap();
        d4 += d5;
        assert_eq!(d4.to_string(), "-140.000000");

        let d6 = Decimal::try_from(140.0).unwrap();
        d4 += d6;
        assert_eq!(d4.to_string(), "0.000000");
        assert_eq!(d4, Decimal::ZERO);
    }

    #[test]
    fn test_decimal_subtraction() {
        let mut d1 = Decimal::try_from(100).unwrap();
        let d2 = Decimal::try_from(50.0).unwrap();
        d1 -= d2;
        assert_eq!(d1.to_string(), "50.000000");

        let d3 = Decimal::try_from(30.0).unwrap();
        d1 -= d3;
        assert_eq!(d1.to_string(), "20.000000");

        let mut d4 = Decimal::try_from(-20.0).unwrap();
        let d5 = Decimal::try_from(-10.0).unwrap();
        d4 -= d5;
        assert_eq!(d4.to_string(), "-10.000000");
        let d6 = Decimal::try_from(-10.0).unwrap();
        d4 -= d6;
        assert_eq!(d4.to_string(), "0.000000");
        assert_eq!(d4, Decimal::ZERO);

        let d7 = Decimal::try_from(100.0).unwrap();
        d4 -= d7;
        assert_eq!(d4.to_string(), "-100.000000");
    }

    #[test]
    fn test_decimal_division() {
        let d1 = Decimal::try_from(100.0).unwrap();
        let d2 = Decimal::try_from(2.0).unwrap();
        let result = d1 / d2;
        assert_eq!(result.to_string(), "50.000000");

        let d3 = Decimal::try_from(0.5).unwrap();
        let result = d1 / d3;
        assert_eq!(result.to_string(), "200.000000");
    }

    #[test]
    #[should_panic(expected = "Division by zero in Decimal division")]
    fn test_decimal_zero_division() {
        let d1 = Decimal::try_from(100.0).unwrap();
        let d2 = Decimal::try_from(0.0).unwrap();
        let _result = d1 / d2; // This should panic
    }

    #[test]
    fn test_decimal_multiplication() {
        let d1 = Decimal::try_from(10.0).unwrap();
        let d2 = Decimal::try_from(5.0).unwrap();
        let result = d1 * d2;
        assert_eq!(result.to_string(), "50.000000");

        let d3 = Decimal::try_from(0.1).unwrap();
        let result = d1 * d3;
        assert_eq!(result.to_string(), "1.000000");

        let d4 = Decimal::try_from(0.0).unwrap();
        let result = d1 * d4;
        assert_eq!(result.to_string(), "0.000000");
        let d5 = Decimal::try_from(-2.0).unwrap();
        let result = d1 * d5;
        assert_eq!(result.to_string(), "-20.000000");

        let d6 = Decimal::try_from(105).unwrap();
        let d7 = Decimal::try_from(0.5).unwrap();
        let result = d6 * d7;
        assert_eq!(result.to_string(), "52.500000");
    }

    #[test]
    fn test_decimal_op_combination() {
        let d1 = Decimal::try_from(100.0).unwrap();
        let d2 = Decimal::try_from(1.5).unwrap();
        assert_eq!((d1 * d2).to_string(), "150.000000");
        let d3 = Decimal::try_from(105).unwrap();
        let d4 = Decimal::try_from(0.5).unwrap();
        assert_eq!((d3 * d4).to_string(), "52.500000");
        assert_eq!(((d1 * d2) + (d3 * d4)).to_string(), "202.500000");
        let d5 = Decimal::try_from(2.0).unwrap();
        let result = ((d1 * d2) + (d3 * d4)) / d5;
        assert_eq!(result.to_string(), "101.250000");
    }
}
