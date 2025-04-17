use crate::*;
// use near_contract_standards::fungible_token::Balance;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, AccountIdRef};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct Stats {
    pub version: String,

    // only onwer can manage accounts and call payments
    pub owner_id: AccountId,

    // token keeped by this vault
    pub token_account_id: AccountId,

    // the static total balance in this vault
    pub total_balance: U128,

    // the start point of linear release
    pub start_timestamp: TimestampSec,

    // the duration of each release round
    pub release_interval: TimestampSec,

    // the total release rounds,
    // we can infer release_per_round = total_balance / release_rounds
    pub release_rounds: u32,

    // already claimed balance, includes account claims and payments
    pub claimed_balance: U128,

    // following are calculated from current env
    pub locked_balance: U128,    // still locked in this vault
    pub liquid_balance: U128,    // liquid balance in this vault
    pub unclaimed_balance: U128, // can be claimed for current
    pub current_round: u32,      // the current release round, start from 1
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct AccountOutput {
    pub account_id: AccountId,
    // the linear release start time point for this account
    pub start_timestamp: TimestampSec,
    // the duration of each claim round for this account
    pub release_interval: TimestampSec,
    // the total rounds of release
    pub release_rounds: u32,
    // the round index of previous claim, start from 1
    pub last_claim_round: u32,
    // total_release = release_rounds * release_per_round
    pub release_per_round: U128,
    // unclaimed amount
    pub unclaimed_amount: U128,
}

#[near_bindgen]
impl Contract {
    pub fn get_stats(&self) -> Stats {
        let (cur_round, unlocked) = self.cur_round_and_total_unlock();
        let (liquid_balance, unclaimed_balance) = self.cur_funding_balance();
        Stats {
            owner_id: self.owner_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            token_account_id: self.token_account_id.clone(),
            total_balance: self.total_balance.into(),
            claimed_balance: self.claimed_balance.into(),
            start_timestamp: self.start_timestamp,
            release_interval: self.release_interval,
            release_rounds: self.release_rounds,
            locked_balance: (self.total_balance.0 - unlocked).into(),
            liquid_balance: liquid_balance.into(),
            unclaimed_balance: unclaimed_balance.into(),
            current_round: cur_round,
        }
    }

    pub fn get_account(&self, account_id: AccountId) -> Option<AccountOutput> {
        let account_ref = AccountIdRef::new(&account_id).expect("ERR_INVALID_ACCOUNT_ID");

        self.accounts.get(account_ref).map(|account| AccountOutput {
            account_id: account.account_id.clone(),
            start_timestamp: account.start_timestamp,
            release_interval: account.release_interval,
            release_rounds: account.release_rounds,
            last_claim_round: account.last_claim_round,
            release_per_round: account.release_per_round,
            unclaimed_amount: account.unclaimed_amount(env::block_timestamp()).into(),
        })
    }

    pub fn list_accounts(&self) -> Vec<AccountOutput> {
        self.accounts
            .values()
            .map(|account| AccountOutput {
                account_id: account.account_id.clone(),
                start_timestamp: account.start_timestamp,
                release_interval: account.release_interval,
                release_rounds: account.release_rounds,
                last_claim_round: account.last_claim_round,
                release_per_round: account.release_per_round.into(),
                unclaimed_amount: account.unclaimed_amount(env::block_timestamp()).into(),
            })
            .collect()
    }
}
