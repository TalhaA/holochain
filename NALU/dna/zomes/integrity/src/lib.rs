use hdi::prelude::*;

#[hdk_entry_helper]
#[derive(Clone)]
pub struct NaluCoin {
    pub owner: AgentPubKey,
    pub balance: u64,
}

#[hdk_entry_helper]
#[derive(Clone)]
pub struct Transaction {
    pub sender: AgentPubKey,
    pub receiver: AgentPubKey,
    pub amount: u64,
    pub timestamp: u64,
}

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    NaluCoin(NaluCoin),
    Transaction(Transaction),
}

#[hdk_link_types]
pub enum LinkTypes {
    AccountLink,
}