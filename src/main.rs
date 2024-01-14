use cosmwasm_std::{Decimal, HumanAddr, Uint128};
use cosmwasm_storage::{ReadonlySingleton, Singleton, ReadonlyBucket, Bucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Balance {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Allowance {
    pub spender: HumanAddr,
    pub owner: HumanAddr,
    pub allowance: Uint128,
}

pub const TOKEN_INFO_KEY: &[u8] = b"token_info";
pub const BALANCES_PREFIX: &[u8] = b"balances";
pub const ALLOWANCES_PREFIX: &[u8] = b"allowances";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    // Add other fields as needed
}

pub struct State {
    pub owner: HumanAddr,
    pub paused: bool,
    pub reentrancy_guard: bool,
    pub token_info: Singleton<TokenInfo>,
    pub balances: Bucket<Balance>,
    pub allowances: Bucket<Allowance>,
}

impl State {
    pub fn new(storage: &mut dyn cosmwasm_std::Storage) -> Self {
        Self {
            token_info: Singleton::new(storage, TOKEN_INFO_KEY),
            balances: Bucket::new(storage, BALANCES_PREFIX),
            allowances: Bucket::new(storage, ALLOWANCES_PREFIX),
            paused: false,
            reentrancy_guard: false,
            owner: HumanAddr::from(""),
        }
    }

    // pub fn save(&mut self, storage: &mut dyn cosmwasm_std::Storage) -> cosmwasm_std::StdResult<()> {
    //     self.token_info.save(storage)?;
    //     Ok(())
    // }

    pub fn readonly(storage: &dyn cosmwasm_std::Storage) -> Self {
        Self {
            token_info: ReadonlySingleton::new(storage, TOKEN_INFO_KEY),
            balances: ReadonlyBucket::new(storage, BALANCES_PREFIX),
            allowances: ReadonlyBucket::new(storage, ALLOWANCES_PREFIX),
            paused: false,
            reentrancy_guard: false,
            owner: HumanAddr::from(""),
        }
    }
}

pub fn pause(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);

    if info.sender != state.owner {
        return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
    }

    state.paused = true;
    state.save(deps.storage)?;

    Ok(cosmwasm_std::Response::new().add_attribute("action", "pause"))
}

pub fn unpause(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);

    if info.sender != state.owner {
        return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
    }

    state.paused = false;
    state.save(deps.storage)?;

    Ok(cosmwasm_std::Response::new().add_attribute("action", "unpause"))
}

// ... instantiate and query functions ...
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub initial_balances: Vec<InitialBalance>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitialBalance {
    pub address: HumanAddr,
    pub amount: Uint128,
}

pub fn instantiate(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
    msg: InstantiateMsg,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let state = State {
        owner: info.sender.clone(),
        balances: HashMap::new(),
        allowances: HashMap::new(),
        paused: false,
        reentrancy_guard: false,
        token_info: TokenInfo {
            name: "My Token".to_string(),
            symbol: "MYT".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        },
    };

    for balance in msg.initial_balances {
        state.balances.insert(balance.address, balance.amount);
    }

    state.save(deps.storage)?;

    Ok(cosmwasm_std::Response::new())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalanceResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalanceQuery {
    pub address: HumanAddr,
}

pub fn query(
    deps: cosmwasm_std::Deps,
    _env: cosmwasm_std::Env,
    msg: BalanceQuery,
) -> cosmwasm_std::StdResult<BalanceResponse> {
    let state = State::new(deps.storage);
    let balance = state.balances.get(&msg.address).unwrap_or(&Uint128::zero());
    Ok(BalanceResponse { amount: *balance })
}

pub fn transfer(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
    recipient: HumanAddr,
    amount: Uint128,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);
    let mut sender_balance = state.balances.load(info.sender.as_bytes())?;
    if sender_balance.amount < amount {
        return Err(cosmwasm_std::StdError::generic_err("Insufficient balance"));
    }
    sender_balance.amount = sender_balance.amount.checked_sub(amount)?;
    state.balances.save(info.sender.as_bytes(), &sender_balance)?;

    let mut recipient_balance = state.balances.load(recipient.as_bytes()).unwrap_or(Balance { amount: Uint128::zero() });
    recipient_balance.amount = recipient_balance.amount.checked_add(amount)?;
    state.balances.save(recipient.as_bytes(), &recipient_balance)?;

    Ok(cosmwasm_std::Response::new().add_attribute("action", "transfer").add_attribute("from", info.sender).add_attribute("to", recipient).add_attribute("amount", amount.to_string()))
}

pub fn approve(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
    spender: HumanAddr,
    amount: Uint128,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);
    let mut allowance = state.allowances.load(&(info.sender.as_bytes().to_vec(), spender.as_bytes().to_vec())).unwrap_or(Allowance { spender: spender.clone(), owner: info.sender.clone(), allowance: Uint128::zero() });
    allowance.allowance = allowance.allowance.checked_add(amount)?;
    state.allowances.save(&(info.sender.as_bytes().to_vec(), spender.as_bytes().to_vec()), &allowance)?;

    Ok(cosmwasm_std::Response::new().add_attribute("action", "approve").add_attribute("owner", info.sender).add_attribute("spender", spender).add_attribute("amount", amount.to_string()))
}

pub fn transfer_from(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
    owner: HumanAddr,
    recipient: HumanAddr,
    amount: Uint128,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);

    // Load the allowance for the spender
    let mut allowance = state.allowances.load(&(owner.as_bytes().to_vec(), info.sender.as_bytes().to_vec())).unwrap_or(Allowance { spender: info.sender.clone(), owner: owner.clone(), allowance: Uint128::zero() });
    if allowance.allowance < amount {
        return Err(cosmwasm_std::StdError::generic_err("Insufficient allowance"));
    }

    // Decrease the allowance
    allowance.allowance = allowance.allowance.checked_sub(amount)?;
    state.allowances.save(&(owner.as_bytes().to_vec(), info.sender.as_bytes().to_vec()), &allowance)?;

    // Load the owner's balance
    let mut owner_balance = state.balances.load(owner.as_bytes())?;
    if owner_balance.amount < amount {
        return Err(cosmwasm_std::StdError::generic_err("Insufficient balance"));
    }

    // Decrease the owner's balance
    owner_balance.amount = owner_balance.amount.checked_sub(amount)?;
    state.balances.save(owner.as_bytes(), &owner_balance)?;

    // Increase the recipient's balance
    let mut recipient_balance = state.balances.load(recipient.as_bytes()).unwrap_or(Balance { amount: Uint128::zero() });
    recipient_balance.amount = recipient_balance.amount.checked_add(amount)?;
    state.balances.save(recipient.as_bytes(), &recipient_balance)?;

    Ok(cosmwasm_std::Response::new().add_attribute("action", "transfer_from").add_attribute("from", owner).add_attribute("to", recipient).add_attribute("amount", amount.to_string()))
}

pub fn decrease_allowance(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
    spender: HumanAddr,
    amount: Uint128,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);

    // Load the allowance for the spender
    let mut allowance = state.allowances.load(&(info.sender.as_bytes().to_vec(), spender.as_bytes().to_vec())).unwrap_or(Allowance { spender: spender.clone(), owner: info.sender.clone(), allowance: Uint128::zero() });
    if allowance.allowance < amount {
        return Err(cosmwasm_std::StdError::generic_err("Insufficient allowance"));
    }

    // Decrease the allowance
    allowance.allowance = allowance.allowance.checked_sub(amount)?;
    state.allowances.save(&(info.sender.as_bytes().to_vec(), spender.as_bytes().to_vec()), &allowance)?;

    Ok(cosmwasm_std::Response::new())
}

pub fn burn(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
    amount: Uint128,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);

    // Load the owner's balance
    let mut owner_balance = state.balances.load(info.sender.as_bytes())?;
    if owner_balance.amount < amount {
        return Err(cosmwasm_std::StdError::generic_err("Insufficient balance"));
    }

    // Decrease the owner's balance
    owner_balance.amount = owner_balance.amount.checked_sub(amount)?;
    state.balances.save(info.sender.as_bytes(), &owner_balance)?;

    Ok(cosmwasm_std::Response::new().add_attribute("action", "burn").add_attribute("from", info.sender).add_attribute("amount", amount.to_string()))
}

pub fn mint(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    info: cosmwasm_std::MessageInfo,
    recipient: HumanAddr,
    amount: Uint128,
) -> cosmwasm_std::StdResult<cosmwasm_std::Response> {
    let mut state = State::new(deps.storage);

    if state.paused {
        return Err(cosmwasm_std::StdError::generic_err("Contract is paused"));
    }

    if state.reentrancy_guard {
        return Err(cosmwasm_std::StdError::generic_err("Reentrant call detected"));
    }

    state.reentrancy_guard = true;
    state.save(deps.storage)?;

    if info.sender != state.owner {
        return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
    }

    // Increase the recipient's balance
    let mut recipient_balance = state.balances.load(recipient.as_bytes()).unwrap_or(Balance { amount: Uint128::zero() });
    recipient_balance.amount = recipient_balance.amount.checked_add(amount)?;
    state.balances.save(recipient.as_bytes(), &recipient_balance)?;

    state.reentrancy_guard = false;
    state.save(deps.storage)?;

    Ok(cosmwasm_std::Response::new().add_attribute("action", "mint").add_attribute("to", recipient).add_attribute("amount", amount.to_string()))
}