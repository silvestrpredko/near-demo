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

const DATA_IMAGE_SVG_TOKEN_B: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='138.627' height='140'%3E%3Cg fill='none' stroke='%23000' stroke-linecap='round' stroke-linejoin='round' stroke-width='4'%3E%3Cellipse cx='69.314' cy='70' rx='67.314' ry='68'/%3E%3Cellipse cx='69.314' cy='70' rx='54.01' ry='54.561'/%3E%3Cpath d='m65.455 36.261-20.068 67.478h44.104l3.749-11.908H63.69l15.878-55.57H65.455zM45.387 77.718l32.196-10.364'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct TokenB {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

#[near_bindgen]
impl TokenB {
    #[init]
    pub fn new_meta_token(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Token B".to_string(),
                symbol: "B$".to_string(),
                icon: Some(DATA_IMAGE_SVG_TOKEN_B.to_string()),
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

near_contract_standards::impl_fungible_token_core!(TokenB, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(TokenB, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for TokenB {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
