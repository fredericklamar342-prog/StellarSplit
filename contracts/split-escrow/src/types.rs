use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SplitStatus {
    Pending,
    Ready,
    Released,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Split {
    pub split_id: u64,
    pub creator: Address,
    pub description: String,
    pub total_amount: i128,
    pub deposited_amount: i128,
    pub status: SplitStatus,
}
