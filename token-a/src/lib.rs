use near_contract_standards::fungible_token::{
    events::FtMint,
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC},
    FungibleToken,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    collections::LazyOption, env, json_types::U128, log, near_bindgen, AccountId, Balance,
    PanicOnDefault, PromiseOrValue,
};

const DATA_IMAGE_SVG_TOKEN_A: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='138.627' height='140'%3E%3Cg fill='none' stroke='%23000' stroke-linecap='round' stroke-linejoin='round' stroke-width='4'%3E%3Cellipse cx='69.314' cy='70' rx='67.314' ry='68'/%3E%3Cellipse cx='69.314' cy='70' rx='54.01' ry='54.561'/%3E%3Cpath d='M78.109 45.807H93.44s14.623 4.39 6.791 22.084c-5.191 11.7-16.726 20.8-30.791 24.651A55.538 55.538 0 0 1 57.532 94.5M57.532 94.5a56.352 56.352 0 0 1-11.144-.428L60.4 60.858M78.109 45.807H41.718M57.532 94.496l-4.893 11.572M82.184 36.151l-4.075 9.656M69.44 92.542l-5.712 13.526M93.272 36.151l-4.074 9.656'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct TokenA {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

#[near_bindgen]
impl TokenA {
    #[init]
    pub fn new_meta_token(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Token A".to_string(),
                symbol: "A$".to_string(),
                icon: Some(DATA_IMAGE_SVG_TOKEN_A.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 10,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut token = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };

        token.token.internal_register_account(&owner_id);
        token.token.internal_deposit(&owner_id, total_supply.into());

        FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();

        token
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(TokenA, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(TokenA, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for TokenA {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
