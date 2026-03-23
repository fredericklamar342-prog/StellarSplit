use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    NotFound = 1,
    AlreadyExists = 2,
    NotAuthorized = 3,
    AlreadyVoted = 4,
    DisputeClosed = 5,
    VotingPeriodActive = 6,
    VotingPeriodEnded = 7,
    InvalidReason = 8,
    SplitNotFound = 9,
}
