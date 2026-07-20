//! Debug-assertion helpers for the CarbonMint contract test suite.
//!
//! This module provides macros that enrich test assertions with detailed
//! diagnostic output so that failures are easier to pinpoint during
//! development.  All helpers are compiled **only** when `#[cfg(test)]` is
//! active (i.e. they are stripped from the release WASM).
//!
//! # Macros
//!
//! | Macro           | Purpose                                       |
//! |-----------------|-----------------------------------------------|
//! | [`assert_ok`]   | Assert a `Result` is `Ok`, printing the error |
//! | [`assert_err`]  | Assert a `Result` matches an expected error   |
//!
//! # Examples
//!
//! ```ignore
//! use carbonmint_contract::{assert_ok, assert_err};
//! use carbonmint_contract::Error;
//!
//! let res: Result<i128, Error> = Ok(42);
//! let val = assert_ok!(res);
//! assert_eq!(val, 42);
//!
//! let res: Result<i128, Error> = Err(Error::Overflow);
//! assert_err!(res, Error::Overflow);
//! ```

/// Asserts that a `Result` is `Ok`, returning the inner value.
///
/// If the value is `Err`, the macro panics with a message that includes the
/// file, line, column, original expression, and the debug representation of
/// the error.  This provides much richer context than a bare `.unwrap()`.
///
/// # Examples
///
/// ```ignore
/// # use carbonmint_contract::{assert_ok, math};
/// let val = assert_ok!(math::checked_add(10, 20));
/// assert_eq!(val, 30);
/// ```
#[macro_export]
#[cfg(test)]
macro_rules! assert_ok {
    ($expr:expr $(,)?) => {{
        match $expr {
            Ok(val) => val,
            Err(e) => {
                panic!(
                    "assert_ok failed: expected Ok(_), got Err({:?})\n  at {}:{}:{}\n  expression: `{}`",
                    e,
                    file!(),
                    line!(),
                    column!(),
                    stringify!($expr),
                );
            }
        }
    }};
}

/// Asserts that a `Result` is `Err` and matches the given pattern.
///
/// The expected error is specifed as a pattern (e.g.
/// `assert_err!(res, Error::Overflow)`).  If the result is `Ok`, or is `Err`
/// but with a *different* variant, the macro panics with a diagnostic message
/// that includes the file, line, column, expression, and both the expected
/// pattern and the actual value.
///
/// # Examples
///
/// ```ignore
/// # use carbonmint_contract::{assert_err, math, Error};
/// let res = math::checked_add(i128::MAX, 1);
/// assert_err!(res, Error::Overflow);
/// ```
#[macro_export]
#[cfg(test)]
macro_rules! assert_err {
    ($expr:expr, $expected:pat $(,)?) => {{
        match $expr {
            Err($expected) => { /* expected */ }
            Err(other) => {
                panic!(
                    "assert_err failed: expected {}, got Err({:?})\n  at {}:{}:{}\n  expression: `{}`",
                    stringify!($expected),
                    other,
                    file!(),
                    line!(),
                    column!(),
                    stringify!($expr),
                );
            }
            Ok(val) => {
                panic!(
                    "assert_err failed: expected Err(_), got Ok({:?})\n  at {}:{}:{}\n  expression: `{}`",
                    val,
                    file!(),
                    line!(),
                    column!(),
                    stringify!($expr),
                );
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::Error;

    // ------------------------------------------------------------------
    // assert_ok!
    // ------------------------------------------------------------------

    #[test]
    fn assert_ok_returns_inner_value() {
        let result: Result<i128, Error> = Ok(42);
        let val = assert_ok!(result);
        assert_eq!(val, 42);
    }

    #[test]
    #[should_panic(expected = "assert_ok failed: expected Ok(_), got Err(Overflow)")]
    fn assert_ok_panics_on_err() {
        let result: Result<i128, Error> = Err(Error::Overflow);
        assert_ok!(result);
    }

    #[test]
    #[should_panic(expected = "assert_ok failed: expected Ok(_), got Err(InvalidAmount)")]
    fn assert_ok_panics_with_correct_error() {
        let result: Result<i128, Error> = Err(Error::InvalidAmount);
        assert_ok!(result);
    }

    #[test]
    fn assert_ok_works_with_trailing_comma() {
        let result: Result<u64, Error> = Ok(7);
        let val = assert_ok!(result,);
        assert_eq!(val, 7);
    }

    // ------------------------------------------------------------------
    // assert_err!
    // ------------------------------------------------------------------

    #[test]
    fn assert_err_matches_expected_variant() {
        let result: Result<i128, Error> = Err(Error::Overflow);
        assert_err!(result, Error::Overflow);
    }

    #[test]
    #[should_panic(
        expected = "assert_err failed: expected Error::AlreadyInitialized, got Err(Overflow)"
    )]
    fn assert_err_panics_on_wrong_variant() {
        let result: Result<i128, Error> = Err(Error::Overflow);
        assert_err!(result, Error::AlreadyInitialized);
    }

    #[test]
    #[should_panic(expected = "assert_err failed: expected Err(_), got Ok(1)")]
    fn assert_err_panics_on_ok() {
        let result: Result<i64, Error> = Ok(1);
        assert_err!(result, Error::Overflow);
    }

    #[test]
    fn assert_err_works_with_trailing_comma() {
        let result: Result<i128, Error> = Err(Error::Paused);
        assert_err!(result, Error::Paused,);
    }

    #[test]
    fn assert_err_matches_any_variant() {
        // Ensures each variant is matchable by the macro.
        let cases: &[Error] = &[
            Error::AlreadyInitialized,
            Error::NotInitialized,
            Error::BatchNotFound,
            Error::InvalidAmount,
            Error::InsufficientBalance,
            Error::Unauthorized,
            Error::Overflow,
            Error::NotListed,
            Error::Paused,
            Error::SameAccount,
        ];
        for &variant in cases {
            let result: Result<i128, Error> = Err(variant);
            assert_err!(result, Error::AlreadyInitialized | Error::NotInitialized | Error::BatchNotFound | Error::InvalidAmount | Error::InsufficientBalance | Error::Unauthorized | Error::Overflow | Error::NotListed | Error::Paused | Error::SameAccount);
        }
    }
}
