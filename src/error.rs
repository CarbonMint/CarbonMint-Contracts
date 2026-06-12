use soroban_sdk::contracterror;

/// Errors that can be returned by the CarbonMint contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The contract has already been initialized.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// No batch exists for the supplied identifier.
    BatchNotFound = 3,
    /// The supplied amount is zero or otherwise invalid.
    InvalidAmount = 4,
    /// The holder does not have enough credits for the operation.
    InsufficientBalance = 5,
    /// The caller is not authorized to perform this action.
    Unauthorized = 6,
    /// An arithmetic operation overflowed.
    Overflow = 7,
}
