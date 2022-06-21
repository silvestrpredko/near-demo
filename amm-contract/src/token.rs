use near_contract_standards::fungible_token::{
    core::FungibleTokenCore, metadata::FungibleTokenMetadata, FungibleToken,
};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    near_bindgen, AccountId, PanicOnDefault,
};

/// Structure that holds a [FungibleToken]
/// Implements basic operations with a token
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Token {
    pub id: AccountId,
    pub internal_token: FungibleToken,
    pub metadata: Option<FungibleTokenMetadata>,
}

impl Token {
    pub fn new(id: AccountId, prefix: Vec<u8>) -> Self {
        Token {
            id,
            internal_token: FungibleToken::new(prefix),
            metadata: None,
        }
    }

    pub fn metadata(&self) -> Option<FungibleTokenMetadata> {
        self.metadata.clone()
    }

    pub fn total_supply(&self) -> U128 {
        self.internal_token.ft_total_supply()
    }

    pub fn balance_of(&self, account_id: AccountId) -> U128 {
        self.internal_token.ft_balance_of(account_id)
    }

    pub fn withdraw(&mut self, account_id: &AccountId, amount: U128) {
        self.internal_token
            .internal_withdraw(account_id, amount.into())
    }

    pub fn transfer(&mut self, sender: &AccountId, receiver: &AccountId, amount: U128) {
        self.internal_token
            .internal_transfer(sender, receiver, amount.into(), None);
    }

    /// Check if account is already registered in the [FungibleToken] storage
    pub fn is_account_registered(&self, account_id: &AccountId) -> bool {
        self.internal_token.accounts.contains_key(account_id)
    }

    /// Try to register an account in [FungibleToken] if it's not registered yet
    /// It's a safe API, it doesn't panic.
    ///
    /// # Examples
    /// ```Rust
    /// // This call could panic if an account isn't registered yet
    /// internal_register_account(account_id);
    /// ```
    pub fn try_register_account(&mut self, account_id: &AccountId) {
        if !self.is_account_registered(account_id) {
            self.internal_token.internal_register_account(account_id);
        }
    }

    pub fn deposit(&mut self, sender_id: &AccountId, amount: U128) {
        self.try_register_account(sender_id);
        self.internal_token
            .internal_deposit(sender_id, amount.into());
    }

    pub fn decimals(&self) -> Option<u8> {
        self.metadata.as_ref().map(|metadata| metadata.decimals)
    }
}

/// Implementation of a simple formula to not transfer more than we have
pub fn calc_transfer_amount(src_pool_balance: U128, dst_pool_balance: U128, amount: U128) -> U128 {
    let portion = src_pool_balance
        .0
        .checked_mul(dst_pool_balance.0)
        .and_then(|it| it.checked_div(src_pool_balance.0 + amount.0))
        .expect("Couldn't calculate a transaction amount");

    dst_pool_balance.0.checked_sub(portion).unwrap().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_transfer_amount() {
        let amount = calc_transfer_amount(6.into(), 30.into(), 1.into());
        assert_eq!(U128::from(5), amount);

        let amount = calc_transfer_amount(6.into(), 30.into(), 2.into());
        assert_eq!(U128::from(8), amount);
    }

    #[test]
    fn test_calc_transfer_amount_from_greater_src() {
        let amount = calc_transfer_amount(100.into(), 20.into(), 50.into());
        assert_eq!(U128::from(7), amount);

        let amount = calc_transfer_amount(100.into(), 20.into(), 10.into());
        assert_eq!(U128::from(2), amount);
    }
}
