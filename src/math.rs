//! Reusable checked and saturating arithmetic helpers.
//!
//! This module provides thin wrappers over Rust's built-in checked and
//! saturating math so that callers don't have to repeat
//! `ok_or(Error::Overflow)` inline.  All functions are panic‑free in
//! accordance with [ADR‑0009](../docs/adr/0009-use-checked-arithmetic-for-all-math.md).
//!
//! # Design decisions
//!
//! | Function                  | Return type               | Use case                              |
//! |---------------------------|---------------------------|---------------------------------------|
//! | [`checked_add`]           | `Result<i128, Error>`     | Operations that **must** not overflow |
//! | [`checked_sub`]           | `Result<i128, Error>`     | Operations that **must** not underflow |
//! | [`checked_mul`]           | `Result<i128, Error>`     | Multiplication that **must** not overflow |
//! | [`checked_div`]           | `Result<i128, Error>`     | Division that **must** not overflow/div-by-zero |
//! | [`saturating_add`]        | `i128`                    | Aggregate counters that can saturate  |
//! | [`saturating_sub`]        | `i128`                    | Aggregate counters that can saturate  |
//! | [`checked_add_u64`]       | `Result<u64, Error>`      | Monotonic id counters                 |
//! | [`saturating_add_u64`]    | `u64`                     | Aggregate `u64` counters              |

use crate::Error;

// ---------------------------------------------------------------------------
// i128 helpers
// ---------------------------------------------------------------------------

/// Checked addition for `i128`.
///
/// Returns `Ok(sum)` on success or [`Error::Overflow`] on overflow.
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::checked_add(10i128, 20i128).unwrap(), 30);
/// assert!(math::checked_add(i128::MAX, 1).is_err());
/// ```
#[inline]
pub fn checked_add(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_add(b).ok_or(Error::Overflow)
}

/// Checked subtraction for `i128`.
///
/// Returns `Ok(diff)` on success or [`Error::Overflow`] on underflow.
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::checked_sub(30i128, 10i128).unwrap(), 20);
/// assert!(math::checked_sub(0i128, 1).is_err());
/// ```
#[inline]
pub fn checked_sub(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_sub(b).ok_or(Error::Overflow)
}

/// Checked multiplication for `i128`.
///
/// Returns `Ok(product)` on success or [`Error::Overflow`] on overflow.
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::checked_mul(10i128, 3i128).unwrap(), 30);
/// assert!(math::checked_mul(i128::MAX, 2).is_err());
/// ```
#[inline]
pub fn checked_mul(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_mul(b).ok_or(Error::Overflow)
}

/// Checked division for `i128`.
///
/// Returns `Ok(quotient)` on success or [`Error::Overflow`] on division by
/// zero or overflow (`i128::MIN / -1`).
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::checked_div(30i128, 3i128).unwrap(), 10);
/// assert!(math::checked_div(1i128, 0).is_err());
/// ```
#[inline]
pub fn checked_div(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_div(b).ok_or(Error::Overflow)
}

/// Saturating addition for `i128`.
///
/// Returns `i128::MAX` / `i128::MIN` instead of overflowing.
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::saturating_add(100i128, 50i128), 150);
/// assert_eq!(math::saturating_add(i128::MAX, 1), i128::MAX);
/// ```
#[inline]
pub fn saturating_add(a: i128, b: i128) -> i128 {
    a.saturating_add(b)
}

/// Saturating subtraction for `i128`.
///
/// Returns `i128::MIN` instead of underflowing.
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::saturating_sub(100i128, 50i128), 50);
/// assert_eq!(math::saturating_sub(0i128, 1), -1);
/// ```
#[inline]
pub fn saturating_sub(a: i128, b: i128) -> i128 {
    a.saturating_sub(b)
}

// ---------------------------------------------------------------------------
// u64 helpers
// ---------------------------------------------------------------------------

/// Checked addition for `u64`.
///
/// Used for monotonic counters (batch ids, certificate ids).
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::checked_add_u64(5u64, 3u64).unwrap(), 8);
/// assert!(math::checked_add_u64(u64::MAX, 1).is_err());
/// ```
#[inline]
pub fn checked_add_u64(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_add(b).ok_or(Error::Overflow)
}

/// Saturating addition for `u64`.
///
/// Clamps to `u64::MAX` instead of wrapping.
///
/// # Examples
///
/// ```ignore
/// use carbonmint_contract::math;
///
/// assert_eq!(math::saturating_add_u64(5u64, 3u64), 8);
/// assert_eq!(math::saturating_add_u64(u64::MAX, 1), u64::MAX);
/// ```
#[inline]
pub fn saturating_add_u64(a: u64, b: u64) -> u64 {
    a.saturating_add(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // checked_add
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_add_ok() {
        assert_eq!(checked_add(10i128, 20i128), Ok(30));
        assert_eq!(checked_add(0i128, 0i128), Ok(0));
        assert_eq!(checked_add(-5i128, 3i128), Ok(-2));
        assert_eq!(checked_add(i128::MAX - 1, 1), Ok(i128::MAX));
        assert_eq!(checked_add(i128::MIN, 1), Ok(i128::MIN + 1));
    }

    #[test]
    fn test_checked_add_overflow() {
        assert_eq!(checked_add(i128::MAX, 1), Err(Error::Overflow));
        assert_eq!(checked_add(i128::MIN, -1), Err(Error::Overflow));
    }

    // -----------------------------------------------------------------------
    // checked_sub
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_sub_ok() {
        assert_eq!(checked_sub(30i128, 10i128), Ok(20));
        assert_eq!(checked_sub(0i128, 0i128), Ok(0));
        assert_eq!(checked_sub(-5i128, -10i128), Ok(5));
        assert_eq!(checked_sub(i128::MIN + 1, 1), Ok(i128::MIN));
    }

    #[test]
    fn test_checked_sub_underflow() {
        assert_eq!(checked_sub(0i128, 1), Err(Error::Overflow));
        assert_eq!(checked_sub(i128::MIN, 1), Err(Error::Overflow));
        assert_eq!(checked_sub(-100i128, i128::MAX), Err(Error::Overflow));
    }

    // -----------------------------------------------------------------------
    // checked_mul
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_mul_ok() {
        assert_eq!(checked_mul(10i128, 3i128), Ok(30));
        assert_eq!(checked_mul(-5i128, 3i128), Ok(-15));
        assert_eq!(checked_mul(0i128, 100i128), Ok(0));
        assert_eq!(checked_mul(1i128, 1i128), Ok(1));
    }

    #[test]
    fn test_checked_mul_overflow() {
        assert_eq!(checked_mul(i128::MAX, 2), Err(Error::Overflow));
        assert_eq!(checked_mul(i128::MIN, 2), Err(Error::Overflow));
    }

    // -----------------------------------------------------------------------
    // checked_div
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_div_ok() {
        assert_eq!(checked_div(30i128, 3i128), Ok(10));
        assert_eq!(checked_div(-30i128, 3i128), Ok(-10));
        assert_eq!(checked_div(0i128, 100i128), Ok(0));
    }

    #[test]
    fn test_checked_div_by_zero() {
        assert_eq!(checked_div(1i128, 0), Err(Error::Overflow));
    }

    #[test]
    fn test_checked_div_overflow() {
        // i128::MIN / -1 overflows because there is no positive 2^127.
        assert_eq!(checked_div(i128::MIN, -1), Err(Error::Overflow));
    }

    // -----------------------------------------------------------------------
    // saturating_add (i128)
    // -----------------------------------------------------------------------
    #[test]
    fn test_saturating_add_normal() {
        assert_eq!(saturating_add(100i128, 50i128), 150);
        assert_eq!(saturating_add(-50i128, 30i128), -20);
        assert_eq!(saturating_add(0i128, 0i128), 0);
    }

    #[test]
    fn test_saturating_add_clamp_positive() {
        assert_eq!(saturating_add(i128::MAX, 1), i128::MAX);
        assert_eq!(saturating_add(i128::MAX, 100), i128::MAX);
    }

    #[test]
    fn test_saturating_add_clamp_negative() {
        assert_eq!(saturating_add(i128::MIN, -1), i128::MIN);
        assert_eq!(saturating_add(i128::MIN, -100), i128::MIN);
    }

    // -----------------------------------------------------------------------
    // saturating_sub (i128)
    // -----------------------------------------------------------------------
    #[test]
    fn test_saturating_sub_normal() {
        assert_eq!(saturating_sub(100i128, 50i128), 50);
        assert_eq!(saturating_sub(0i128, 0i128), 0);
        assert_eq!(saturating_sub(-10i128, -5i128), -5);
    }

    #[test]
    fn test_saturating_sub_clamp_negative() {
        assert_eq!(saturating_sub(0i128, 1), -1);
        assert_eq!(saturating_sub(i128::MIN, 1), i128::MIN);
    }

    // -----------------------------------------------------------------------
    // checked_add_u64
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_add_u64_ok() {
        assert_eq!(checked_add_u64(5u64, 3u64), Ok(8));
        assert_eq!(checked_add_u64(0u64, 0u64), Ok(0));
        assert_eq!(checked_add_u64(u64::MAX - 1, 1), Ok(u64::MAX));
    }

    #[test]
    fn test_checked_add_u64_overflow() {
        assert_eq!(checked_add_u64(u64::MAX, 1), Err(Error::Overflow));
    }

    // -----------------------------------------------------------------------
    // saturating_add_u64
    // -----------------------------------------------------------------------
    #[test]
    fn test_saturating_add_u64_normal() {
        assert_eq!(saturating_add_u64(5u64, 3u64), 8);
        assert_eq!(saturating_add_u64(0u64, 0u64), 0);
    }

    #[test]
    fn test_saturating_add_u64_clamp() {
        assert_eq!(saturating_add_u64(u64::MAX, 1), u64::MAX);
        assert_eq!(saturating_add_u64(u64::MAX, 100), u64::MAX);
    }
}
