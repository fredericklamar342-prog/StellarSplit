use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidFeeBps = 5,
    SplitNotFound = 6,
    SplitNotPending = 7,
    SplitNotReady = 8,
    TreasuryNotSet = 9,
    ParticipantCapExceeded = 10,
    // Money-critical/upgrade failures are asserted by numeric code in tests.
    InvalidVersion = 11,
    EscrowNotActive = 12,
    InvalidMetadata = 13,
    SplitNotActive = 14,
    InvalidInput = 15,
}
