mod token;

use near_contract_standards::fungible_token::{
    metadata::FungibleTokenMetadata, receiver::FungibleTokenReceiver,
};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    AccountId, PanicOnDefault, Promise, PromiseOrValue, PromiseResult,
};
use near_sdk::{env, ext_contract, log, near_bindgen};
use serde::{Deserialize, Serialize};
use token::Token;

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn set_token_metadata(&mut self, token_type: TokenType);
    fn withdraw_token_callback(
        &mut self,
        token_id: AccountId,
        user_account_id: AccountId,
        amount: U128,
    );
}

#[ext_contract(ext_ft)]
pub trait FtToken {
    fn ft_metadata() -> FungibleTokenMetadata;
    fn ft_transfer(receiver_id: AccountId, amount: U128);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AmmContract {
    owner_id: AccountId,
    token_a: Token,
    token_b: Token,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TokenType {
    A,
    B,
}

#[near_bindgen]
impl AmmContract {
    #[init]
    pub fn initialize(owner_id: AccountId, token_a_id: AccountId, token_b_id: AccountId) -> Self {
        // Let's fetch a metadata for a provided tokens
        metadata(token_a_id.clone(), TokenType::A);
        metadata(token_b_id.clone(), TokenType::B);

        Self {
            owner_id,
            token_a: Token::new(token_a_id, b"a".to_vec()),
            token_b: Token::new(token_b_id, b"b".to_vec()),
        }
    }

    pub fn add_liquidity(
        &mut self,
        token_a_id: AccountId,
        amount_liq_a: U128,
        token_b_id: AccountId,
        amount_liq_b: U128,
    ) {
        // Supports only known tokens
        if token_a_id != self.token_a.id || token_b_id != self.token_b.id {
            env::panic_str(
                format!(
                    "Passed token_a_id {token_a_id} and token_b_id {token_b_id} \
                    doesn't match with previously initialized respectively {} and {}",
                    self.token_a.id, self.token_b.id
                )
                .as_str(),
            )
        }

        let contract_id = env::current_account_id();
        let owner_id = env::predecessor_account_id();
        let signer_account_id = env::signer_account_id();

        if self.owner_id != owner_id && self.owner_id != signer_account_id {
            env::panic_str("Access unauthorized");
        }

        let token_a_balance = self.token_a.balance_of(owner_id.clone());
        let token_b_balance = self.token_b.balance_of(owner_id.clone());
        if token_a_balance < amount_liq_a || token_b_balance < amount_liq_b {
            env::panic_str(
                format!(
                    "Not enough balance to add liquidity, \
                    Token A balance: {token_a_balance:?}, \
                    Token B balance: {token_b_balance:?}"
                )
                .as_str(),
            );
        }

        let liq_balance_a = self.token_a.balance_of(contract_id.clone());
        let liq_balance_b = self.token_b.balance_of(contract_id.clone());

        if liq_balance_a == 0.into() && liq_balance_b == 0.into() {
            // At the first call, tokens don't have a contract accounts
            self.token_a.try_register_account(&contract_id);
            self.token_b.try_register_account(&contract_id);
            self.token_a.transfer(&owner_id, &contract_id, amount_liq_a);
            self.token_b.transfer(&owner_id, &contract_id, amount_liq_b);
        } else {
            let exchange_rate = liq_balance_a.0.checked_div(liq_balance_b.0).unwrap();
            let amount_check = amount_liq_a.0.checked_mul(exchange_rate).unwrap();

            if amount_liq_b > amount_check.into() {
                env::panic_str("Incorrect amounts for top up a liquidity")
            }

            self.token_a.transfer(&owner_id, &contract_id, amount_liq_a);
            self.token_b.transfer(&owner_id, &contract_id, amount_liq_b);
        }
    }

    pub fn swap(&mut self, from_token_id: AccountId, to_token_id: AccountId, amount: U128) {
        let contract_id = env::current_account_id();
        let user_account_id = env::predecessor_account_id();

        if self
            .token(&from_token_id)
            .balance_of(user_account_id.clone())
            < amount
        {
            env::panic_str(
                format!("The user {} doesn't have enough funds", &user_account_id).as_str(),
            );
        }

        // Get current statement of pool
        let src_pool_balance = self.token(&from_token_id).balance_of(contract_id.clone());
        let dst_pool_balance = self.token(&to_token_id).balance_of(contract_id.clone());

        if src_pool_balance == 0.into() || dst_pool_balance == 0.into() {
            env::panic_str("Pool balance couldn't be equal to 0");
        }

        self.token(&from_token_id)
            .transfer(&user_account_id, &contract_id, amount);

        let amount_to_transfer =
            token::calc_transfer_amount(src_pool_balance, dst_pool_balance, amount);

        // In case if other token wallet not used yet
        self.token(&to_token_id)
            .try_register_account(&user_account_id);
        self.token(&to_token_id)
            .transfer(&contract_id, &user_account_id, amount_to_transfer);
    }

    #[payable]
    pub fn withdraw_token(&mut self, token_id: AccountId, amount: U128) -> Promise {
        let user_account_id = env::predecessor_account_id();
        let user_balance = self.token(&token_id).balance_of(user_account_id.clone());

        if user_balance < amount {
            env::panic_str(
                format!(
                    "The user doesn't hold so many funds {amount:?}, \
                    User Balance is {user_balance:?}"
                )
                .as_str(),
            )
        }

        ext_ft::ext(token_id.clone())
            .with_attached_deposit(1)
            .ft_transfer(user_account_id.clone(), amount)
            .then(
                ext_self::ext(env::current_account_id()).withdraw_token_callback(
                    token_id,
                    user_account_id,
                    amount,
                ),
            )
    }

    pub fn token_a_meta(&self) -> FungibleTokenMetadata {
        self.token_a
            .metadata()
            .unwrap_or_else(|| env::panic_str("Metadata for a token A is empty"))
    }

    pub fn token_b_meta(&self) -> FungibleTokenMetadata {
        self.token_b
            .metadata()
            .unwrap_or_else(|| env::panic_str("Metadata for a token B is empty"))
    }

    pub fn token_a_supply(&self) -> U128 {
        self.token_a.total_supply()
    }

    pub fn token_a_in_pool(&self) -> U128 {
        self.token_a.balance_of(env::current_account_id())
    }

    pub fn token_b_supply(&self) -> U128 {
        self.token_b.total_supply()
    }

    pub fn token_b_in_pool(&self) -> U128 {
        self.token_b.balance_of(env::current_account_id())
    }

    pub fn balance_of_token_a(&self, account_id: AccountId) -> U128 {
        self.token_a.balance_of(account_id)
    }

    pub fn balance_of_token_b(&self, account_id: AccountId) -> U128 {
        self.token_b.balance_of(account_id)
    }

    fn token(&mut self, token_id: &AccountId) -> &mut Token {
        match token_id {
            id if *id == self.token_a.id => &mut self.token_a,
            id if *id == self.token_b.id => &mut self.token_b,
            _ => env::panic_str(format!("Doesn't support passed token_id {token_id}").as_str()),
        }
    }

    #[private]
    pub fn set_token_metadata(&mut self, token_type: TokenType) {
        assert_eq!(env::promise_results_count(), 1, "Expected 1 promise result");
        match env::promise_result(0) {
            PromiseResult::NotReady => env::panic_str("Metadata promise isn't ready"),
            PromiseResult::Successful(data) => {
                let meta: Option<FungibleTokenMetadata> =
                    near_sdk::serde_json::from_slice(&data).ok();
                match token_type {
                    TokenType::A => self.token_a.metadata = meta,
                    TokenType::B => self.token_b.metadata = meta,
                }
            }
            PromiseResult::Failed => env::panic_str(
                format!("Couldn't set a metadata for a token {token_type:?}").as_str(),
            ),
        }
    }

    #[private]
    pub fn withdraw_token_callback(
        &mut self,
        token_id: AccountId,
        user_account_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(env::promise_results_count(), 1, "Expected 1 promise result");
        match env::promise_result(0) {
            PromiseResult::NotReady => env::panic_str("Token withdraw callback not ready"),
            PromiseResult::Successful(_) => {
                self.token(&token_id).withdraw(&user_account_id, amount);
            }
            PromiseResult::Failed => env::panic_str("Token withdraw failed"),
        }
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for AmmContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        #[allow(unused_variables)] msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = env::predecessor_account_id();
        if self.token_a.id == token_id {
            self.token_a.deposit(&sender_id, amount);
            PromiseOrValue::Value(0.into())
        } else if self.token_b.id == token_id {
            self.token_b.deposit(&sender_id, amount);
            PromiseOrValue::Value(0.into())
        } else {
            log!("Doesn't support such token");
            PromiseOrValue::Value(amount)
        }
    }
}

fn metadata(token_id: AccountId, token_type: TokenType) -> Promise {
    ext_ft::ext(token_id)
        .ft_metadata()
        .then(ext_self::ext(env::current_account_id()).set_token_metadata(token_type))
}
