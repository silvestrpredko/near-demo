mod api;

use anyhow::Ok;
use near_sdk::json_types::U128;
use near_units::parse_near;
use workspaces::prelude::*;
use workspaces::{network::Sandbox, Account, Contract, Worker};

const WASM_FILEPATH_CONTRACT: &str = "target/wasm32-unknown-unknown/release/amm_contract.wasm";
const WASM_FILEPATH_TOKEN_A: &str = "target/wasm32-unknown-unknown/release/token_a.wasm";
const WASM_FILEPATH_TOKEN_B: &str = "target/wasm32-unknown-unknown/release/token_b.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;

    // deploy contracts
    let contract_wasm = std::fs::read(WASM_FILEPATH_CONTRACT)?;
    let amm_contract = worker.dev_deploy(&contract_wasm).await?;
    let token_a_wasm = std::fs::read(WASM_FILEPATH_TOKEN_A)?;
    let token_a_contract = worker.dev_deploy(&token_a_wasm).await?;
    let token_b_wasm = std::fs::read(WASM_FILEPATH_TOKEN_B)?;
    let token_b_contract = worker.dev_deploy(&token_b_wasm).await?;

    // create accounts
    let owner = worker.root_account();

    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // Init tokens
    token_a_contract
        .call(&worker, "new_meta_token")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))?
        .transact()
        .await?;

    token_b_contract
        .call(&worker, "new_meta_token")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))?
        .transact()
        .await?;

    // Init amm contract
    amm_contract
        .call(&worker, "initialize")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "token_a_id": token_a_contract.id(),
            "token_b_id": token_b_contract.id(),
        }))?
        .max_gas()
        .transact()
        .await?;

    storage_deposits(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &alice,
    )
    .await?;

    // begin tests
    // keep the execution sequence
    test_add_liquidity_without_enough_balance(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &owner,
    )
    .await?;
    test_add_liquidity(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &owner,
        &alice,
    )
    .await?;
    test_swap_wrong_amount(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &owner,
        &alice,
    )
    .await?;
    test_swap_correct_amount(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &alice,
    )
    .await?;
    test_token_withdraw(&worker, &amm_contract, &token_a_contract, &alice).await?;
    test_add_liquidity_wrong_again(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &owner,
        &alice,
    )
    .await?;
    test_add_liquidity_correct_again(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &owner,
        &alice,
    )
    .await?;
    test_add_liquidity_with_wrong_owner(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &alice,
    )
    .await?;
    test_add_liquidity_with_wrong_tokens(
        &worker,
        &amm_contract,
        &token_a_contract,
        &token_b_contract,
        &owner,
    )
    .await?;

    Ok(())
}

async fn test_add_liquidity_without_enough_balance(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    owner: &Account,
) -> anyhow::Result<()> {
    let res = api::add_liquidity(
        worker,
        owner,
        amm_contract,
        token_a_contract.id(),
        30.into(),
        token_b_contract.id(),
        6.into(),
    )
    .await;

    matches!(res, Result::Err(err) if err.to_string().as_str().starts_with("Not enough balance to add liquidity"));
    println!("      Passed ✅ test_add_liquidity_without_enough_balance");
    Ok(())
}

async fn test_add_liquidity(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    owner: &Account,
    alice: &Account,
) -> anyhow::Result<()> {
    api::ft_transfer_call(
        worker,
        token_a_contract,
        owner,
        amm_contract.as_account(),
        30.into(),
    )
    .await?;

    api::ft_transfer_call(
        worker,
        token_b_contract,
        owner,
        amm_contract.as_account(),
        6.into(),
    )
    .await?;

    api::add_liquidity(
        worker,
        owner,
        amm_contract,
        token_a_contract.id(),
        30.into(),
        token_b_contract.id(),
        6.into(),
    )
    .await?;

    let amount_token_a = api::token_a_in_pool(worker, amm_contract, alice).await?;
    let amount_token_b = api::token_b_in_pool(worker, amm_contract, alice).await?;

    assert_eq!(U128::from(30), amount_token_a);
    assert_eq!(U128::from(6), amount_token_b);

    println!("      Passed ✅ test_add_liquidity");
    Ok(())
}

async fn test_swap_wrong_amount(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    owner: &Account,
    alice: &Account,
) -> anyhow::Result<()> {
    // Top up a user account
    api::ft_transfer(worker, token_a_contract, owner, alice, 20.into()).await?;
    api::ft_transfer(worker, token_b_contract, owner, alice, 20.into()).await?;

    api::ft_transfer_call(
        worker,
        token_b_contract,
        alice,
        amm_contract.as_account(),
        1.into(),
    )
    .await?;

    let res = api::swap(
        worker,
        amm_contract,
        alice,
        token_b_contract.id(),
        token_a_contract.id(),
        5.into(),
    )
    .await;

    assert!(res.is_err());

    println!("      Passed ✅ test_swap_wrong_amount");
    Ok(())
}

async fn test_swap_correct_amount(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    alice: &Account,
) -> anyhow::Result<()> {
    api::swap(
        worker,
        amm_contract,
        alice,
        token_b_contract.id(),
        token_a_contract.id(),
        1.into(),
    )
    .await?;

    let user_balance = api::balance_of_token_a(worker, amm_contract, alice).await?;
    assert_eq!(U128::from(5), user_balance);

    println!("      Passed ✅ test_swap_correct_amount");
    Ok(())
}

async fn test_token_withdraw(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    alice: &Account,
) -> anyhow::Result<()> {
    api::withdraw_token(worker, amm_contract, alice, token_a_contract.id(), 5.into()).await?;
    let balance = api::ft_balance_of(worker, alice, token_a_contract.id()).await?;

    assert_eq!(Into::<U128>::into(25), balance);
    println!("      Passed ✅ test_token_withdraw");
    Ok(())
}

async fn test_add_liquidity_wrong_again(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    owner: &Account,
    alice: &Account,
) -> anyhow::Result<()> {
    // Check pool after swap
    let amount_token_a = api::token_a_in_pool(worker, amm_contract, alice).await?;
    let amount_token_b = api::token_b_in_pool(worker, amm_contract, alice).await?;

    assert_eq!(U128::from(25), amount_token_a);
    assert_eq!(U128::from(7), amount_token_b);

    api::ft_transfer_call(
        worker,
        token_a_contract,
        owner,
        amm_contract.as_account(),
        50.into(),
    )
    .await?;

    api::ft_transfer_call(
        worker,
        token_b_contract,
        owner,
        amm_contract.as_account(),
        50.into(),
    )
    .await?;

    let res = api::add_liquidity(
        worker,
        owner,
        amm_contract,
        token_a_contract.id(),
        5.into(),
        token_b_contract.id(),
        26.into(),
    )
    .await;

    matches!(res, Result::Err(err) if err.to_string().as_str() == "Incorrect amounts for top up a liquidity");
    println!("      Passed ✅ test_add_liquidity_wrong_again");

    Ok(())
}

async fn test_add_liquidity_correct_again(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    owner: &Account,
    alice: &Account,
) -> anyhow::Result<()> {
    api::add_liquidity(
        worker,
        owner,
        amm_contract,
        token_a_contract.id(),
        5.into(),
        token_b_contract.id(),
        15.into(),
    )
    .await?;

    // Check pool after swap
    let amount_token_a = api::token_a_in_pool(worker, amm_contract, alice).await?;
    let amount_token_b = api::token_b_in_pool(worker, amm_contract, alice).await?;

    assert_eq!(U128::from(30), amount_token_a);
    assert_eq!(U128::from(22), amount_token_b);

    println!("      Passed ✅ test_add_liquidity_correct_again");
    Ok(())
}

async fn test_add_liquidity_with_wrong_owner(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    alice: &Account,
) -> anyhow::Result<()> {
    let res = api::add_liquidity(
        worker,
        alice,
        amm_contract,
        token_a_contract.id(),
        30.into(),
        token_b_contract.id(),
        6.into(),
    )
    .await;

    matches!(res, Result::Err(err) if err.to_string().as_str() == "Access unauthorized");
    println!("      Passed ✅ test_add_liquidity_with_wrong_owner");
    Ok(())
}

async fn test_add_liquidity_with_wrong_tokens(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    owner: &Account,
) -> anyhow::Result<()> {
    let res = api::add_liquidity(
        worker,
        owner,
        amm_contract,
        token_b_contract.id(),
        30.into(),
        token_a_contract.id(),
        6.into(),
    )
    .await;

    assert!(res.is_err());

    println!("      Passed ✅ test_add_liquidity_with_wrong_tokens");
    Ok(())
}

async fn storage_deposits(
    worker: &Worker<Sandbox>,
    amm_contract: &Contract,
    token_a_contract: &Contract,
    token_b_contract: &Contract,
    alice: &Account,
) -> anyhow::Result<()> {
    api::storage_deposit(
        worker,
        token_a_contract,
        amm_contract.as_account(),
        parse_near!("1 N"),
    )
    .await?;

    api::storage_deposit(
        worker,
        token_b_contract,
        amm_contract.as_account(),
        parse_near!("1 N"),
    )
    .await?;

    api::storage_deposit(worker, token_a_contract, alice, parse_near!("1 N")).await?;
    api::storage_deposit(worker, token_b_contract, alice, parse_near!("1 N")).await?;

    Ok(())
}
