use near_sdk::json_types::U128;
use near_units::parse_near;
use workspaces::{network::Sandbox, Account, AccountId, Contract, Worker};

pub async fn storage_deposit(
    worker: &Worker<Sandbox>,
    token_contract: &Contract,
    account: &Account,
    deposit: u128,
) -> anyhow::Result<()> {
    account
        .call(worker, token_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({ "account_id": account.id() }))?
        .deposit(deposit)
        .transact()
        .await?;
    Ok(())
}

pub async fn ft_transfer(
    worker: &Worker<Sandbox>,
    token_contract: &Contract,
    from: &Account,
    to: &Account,
    amount: U128,
) -> anyhow::Result<()> {
    from.call(worker, token_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({"receiver_id": to.id(), "amount": amount}))?
        .deposit(1)
        .transact()
        .await?;
    Ok(())
}

pub async fn ft_balance_of(
    worker: &Worker<Sandbox>,
    user: &Account,
    token_id: &AccountId,
) -> anyhow::Result<U128> {
    user.call(worker, token_id, "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .transact()
        .await?
        .json()
}

pub async fn ft_transfer_call(
    worker: &Worker<Sandbox>,
    token_contract: &Contract,
    from: &Account,
    to: &Account,
    amount: U128,
) -> anyhow::Result<()> {
    from.call(worker, token_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({"receiver_id": to.id(), "amount": amount, "msg": "0"}))?
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    Ok(())
}

pub async fn add_liquidity(
    worker: &Worker<Sandbox>,
    owner: &Account,
    amm_contract: &Contract,
    token_a_id: &AccountId,
    amount_liq_a: U128,
    token_b_id: &AccountId,
    amount_liq_b: U128,
) -> anyhow::Result<()> {
    owner
        .call(worker, amm_contract.id(), "add_liquidity")
        .args_json(serde_json::json!({
            "token_a_id": token_a_id,
            "amount_liq_a": amount_liq_a,
            "token_b_id": token_b_id,
            "amount_liq_b": amount_liq_b,
        }))?
        .max_gas()
        .transact()
        .await?;
    Ok(())
}

pub async fn swap(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
    from: &AccountId,
    to: &AccountId,
    amount: U128,
) -> anyhow::Result<()> {
    user.call(worker, amm_contract.id(), "swap")
        .args_json(serde_json::json!({
            "from_token_id": from,
            "to_token_id": to,
            "amount": amount,
        }))?
        .max_gas()
        .transact()
        .await?;
    Ok(())
}

pub async fn withdraw_token(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
    token_id: &AccountId,
    amount: U128,
) -> anyhow::Result<()> {
    user.call(worker, amm_contract.id(), "withdraw_token")
        .args_json(serde_json::json!({
            "token_id": token_id,
            "amount": amount,
        }))?
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn token_a_supply(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
) -> anyhow::Result<U128> {
    user.call(worker, amm_contract.id(), "token_a_supply")
        .max_gas()
        .transact()
        .await?
        .json()
}

#[allow(dead_code)]
pub async fn token_b_supply(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
) -> anyhow::Result<U128> {
    user.call(worker, amm_contract.id(), "token_b_supply")
        .max_gas()
        .transact()
        .await?
        .json()
}

pub async fn token_a_in_pool(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
) -> anyhow::Result<U128> {
    user.call(worker, amm_contract.id(), "token_a_in_pool")
        .max_gas()
        .transact()
        .await?
        .json()
}

pub async fn token_b_in_pool(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
) -> anyhow::Result<U128> {
    user.call(worker, amm_contract.id(), "token_b_in_pool")
        .max_gas()
        .transact()
        .await?
        .json()
}

#[allow(dead_code)]
pub async fn balance_of_token_b(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
) -> anyhow::Result<U128> {
    user.call(worker, amm_contract.id(), "balance_of_token_b")
        .args_json(serde_json::json!({"account_id": user.id()}))?
        .max_gas()
        .transact()
        .await?
        .json()
}

pub async fn balance_of_token_a(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    user: &Account,
) -> anyhow::Result<U128> {
    user.call(worker, amm_contract.id(), "balance_of_token_a")
        .args_json(serde_json::json!({"account_id": user.id()}))?
        .max_gas()
        .transact()
        .await?
        .json()
}
