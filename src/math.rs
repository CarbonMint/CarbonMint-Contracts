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
    use crate::{assert_err, assert_ok};

    // -----------------------------------------------------------------------
    // checked_add
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_add_ok() {
        assert_eq!(assert_ok!(checked_add(10i128, 20i128)), 30);
        assert_eq!(assert_ok!(checked_add(0i128, 0i128)), 0);
        assert_eq!(assert_ok!(checked_add(-5i128, 3i128)), -2);
        assert_eq!(assert_ok!(checked_add(i128::MAX - 1, 1)), i128::MAX);
        assert_eq!(assert_ok!(checked_add(i128::MIN, 1)), i128::MIN + 1);
    }

    #[test]
    fn test_checked_add_overflow() {
        assert_err!(checked_add(i128::MAX, 1), Error::Overflow);
        assert_err!(checked_add(i128::MIN, -1), Error::Overflow);
    }

    // -----------------------------------------------------------------------
    // checked_sub
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_sub_ok() {
        assert_eq!(assert_ok!(checked_sub(30i128, 10i128)), 20);
        assert_eq!(assert_ok!(checked_sub(0i128, 0i128)), 0);
        assert_eq!(assert_ok!(checked_sub(-5i128, -10i128)), 5);
        assert_eq!(assert_ok!(checked_sub(i128::MIN + 1, 1)), i128::MIN);
    }

    #[test]
    fn test_checked_sub_underflow() {
        // i128::MIN - 1 wraps to i128::MAX, so checked_sub must reject it.
        assert_err!(checked_sub(i128::MIN, 1), Error::Overflow);
        // -100 - i128::MAX underflows the i128 range.
        assert_err!(checked_sub(-100i128, i128::MAX), Error::Overflow);
    }

    // -----------------------------------------------------------------------
    // checked_mul
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_mul_ok() {
        assert_eq!(assert_ok!(checked_mul(10i128, 3i128)), 30);
        assert_eq!(assert_ok!(checked_mul(-5i128, 3i128)), -15);
        assert_eq!(assert_ok!(checked_mul(0i128, 100i128)), 0);
        assert_eq!(assert_ok!(checked_mul(1i128, 1i128)), 1);
    }

    #[test]
    fn test_checked_mul_overflow() {
        assert_err!(checked_mul(i128::MAX, 2), Error::Overflow);
        assert_err!(checked_mul(i128::MIN, 2), Error::Overflow);
    }

    // -----------------------------------------------------------------------
    // checked_div
    // -----------------------------------------------------------------------
    #[test]
    fn test_checked_div_ok() {
        assert_eq!(assert_ok!(checked_div(30i128, 3i128)), 10);
        assert_eq!(assert_ok!(checked_div(-30i128, 3i128)), -10);
        assert_eq!(assert_ok!(checked_div(0i128, 100i128)), 0);
    }

    #[test]
    fn test_checked_div_by_zero() {
        assert_err!(checked_div(1i128, 0), Error::Overflow);
    }

    #[test]
    fn test_checked_div_overflow() {
        // i128::MIN / -1 overflows because there is no positive 2^127.
        assert_err!(checked_div(i128::MIN, -1), Error::Overflow);
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
        assert_eq!(assert_ok!(checked_add_u64(5u64, 3u64)), 8);
        assert_eq!(assert_ok!(checked_add_u64(0u64, 0u64)), 0);
        assert_eq!(assert_ok!(checked_add_u64(u64::MAX - 1, 1)), u64::MAX);
    }

    #[test]
    fn test_checked_add_u64_overflow() {
        assert_err!(checked_add_u64(u64::MAX, 1), Error::Overflow);
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

    // -----------------------------------------------------------------------
    // Maximum-value arithmetic paths
    //
    // These cases exercise every helper at the numeric ceiling of its type
    // to confirm that identity operations, near-boundary subtractions, and
    // zero-multiplications all return Ok rather than Err(Overflow).
    // -----------------------------------------------------------------------

    /// checked_add: adding 0 to i128::MAX must return Ok(i128::MAX).
    #[test]
    fn test_checked_add_max_plus_zero() {
        assert_eq!(checked_add(i128::MAX, 0).unwrap(), i128::MAX);
    }

    /// checked_add: adding i128::MAX - 1 to 1 must return Ok(i128::MAX).
    #[test]
    fn test_checked_add_reaches_max() {
        assert_eq!(checked_add(i128::MAX - 1, 1).unwrap(), i128::MAX);
    }

    /// checked_sub: i128::MAX - i128::MAX must return Ok(0).
    #[test]
    fn test_checked_sub_max_minus_max() {
        assert_eq!(checked_sub(i128::MAX, i128::MAX).unwrap(), 0);
    }

    /// checked_sub: subtracting 0 from i128::MAX must return Ok(i128::MAX).
    #[test]
    fn test_checked_sub_max_minus_zero() {
        assert_eq!(checked_sub(i128::MAX, 0).unwrap(), i128::MAX);
    }

    /// checked_sub: i128::MIN - 0 must return Ok(i128::MIN).
    #[test]
    fn test_checked_sub_min_minus_zero() {
        assert_eq!(checked_sub(i128::MIN, 0).unwrap(), i128::MIN);
    }

    /// checked_mul: i128::MAX × 1 is the identity and must return Ok(i128::MAX).
    #[test]
    fn test_checked_mul_max_by_one() {
        assert_eq!(checked_mul(i128::MAX, 1).unwrap(), i128::MAX);
    }

    /// checked_mul: i128::MAX × 0 must return Ok(0) (zero-product).
    #[test]
    fn test_checked_mul_max_by_zero() {
        assert_eq!(checked_mul(i128::MAX, 0).unwrap(), 0);
    }

    /// checked_mul: i128::MIN × 1 must return Ok(i128::MIN).
    #[test]
    fn test_checked_mul_min_by_one() {
        assert_eq!(checked_mul(i128::MIN, 1).unwrap(), i128::MIN);
    }

    /// checked_mul: i128::MIN × 0 must return Ok(0).
    #[test]
    fn test_checked_mul_min_by_zero() {
        assert_eq!(checked_mul(i128::MIN, 0).unwrap(), 0);
    }

    /// checked_div: i128::MAX / 1 must return Ok(i128::MAX) (identity divisor).
    #[test]
    fn test_checked_div_max_by_one() {
        assert_eq!(checked_div(i128::MAX, 1).unwrap(), i128::MAX);
    }

    /// checked_div: i128::MAX / i128::MAX must return Ok(1).
    #[test]
    fn test_checked_div_max_by_max() {
        assert_eq!(checked_div(i128::MAX, i128::MAX).unwrap(), 1);
    }

    /// checked_div: i128::MAX / -1 must return Ok(-i128::MAX) (no overflow;
    /// only i128::MIN / -1 overflows because -i128::MIN is not representable).
    #[test]
    fn test_checked_div_max_by_neg_one() {
        assert_eq!(checked_div(i128::MAX, -1).unwrap(), -i128::MAX);
    }

    /// checked_div: 0 / i128::MAX must return Ok(0).
    #[test]
    fn test_checked_div_zero_by_max() {
        assert_eq!(checked_div(0, i128::MAX).unwrap(), 0);
    }

    /// saturating_add (i128): MAX + 0 must return MAX (not clamp unnecessarily).
    #[test]
    fn test_saturating_add_max_plus_zero() {
        assert_eq!(saturating_add(i128::MAX, 0), i128::MAX);
    }

    /// saturating_sub (i128): MIN - 0 must return MIN.
    #[test]
    fn test_saturating_sub_min_minus_zero() {
        assert_eq!(saturating_sub(i128::MIN, 0), i128::MIN);
    }

    /// saturating_sub (i128): MAX - MAX must return 0.
    #[test]
    fn test_saturating_sub_max_minus_max() {
        assert_eq!(saturating_sub(i128::MAX, i128::MAX), 0);
    }

    /// saturating_sub (i128): saturating below MIN must clamp to MIN.
    #[test]
    fn test_saturating_sub_clamp_to_min() {
        assert_eq!(saturating_sub(i128::MIN, 1), i128::MIN);
        assert_eq!(saturating_sub(i128::MIN, i128::MAX), i128::MIN);
    }

    /// checked_add_u64: u64::MAX + 0 must return Ok(u64::MAX).
    #[test]
    fn test_checked_add_u64_max_plus_zero() {
        assert_eq!(checked_add_u64(u64::MAX, 0).unwrap(), u64::MAX);
    }

    /// saturating_add_u64: u64::MAX + 0 must return u64::MAX (no clamp needed).
    #[test]
    fn test_saturating_add_u64_max_plus_zero() {
        assert_eq!(saturating_add_u64(u64::MAX, 0), u64::MAX);
    }

    /// saturating_add_u64: any overflow must clamp to u64::MAX, not wrap.
    #[test]
    fn test_saturating_add_u64_overflow_clamps() {
        assert_eq!(saturating_add_u64(u64::MAX, u64::MAX), u64::MAX);
    }
}
