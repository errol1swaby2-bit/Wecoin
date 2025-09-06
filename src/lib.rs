use pyo3::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Serialize, Deserialize};
use serde_json::json;
use chrono::Utc;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};

/// Pool kinds as string keys
const POOL_TREASURY: &str = "treasury";
const POOL_JURORS: &str = "jurors";
const POOL_CREATORS: &str = "creators";
const POOL_OPERATORS: &str = "operators";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Event {
    pub tag: String,
    pub details: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Clone)]
struct Config {
    treasury_reward: u128,
    jurors_reward: u128,
    creators_reward: u128,
    operators_reward: u128,
    cooldown_epochs: u64,
    max_supply: u128,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            treasury_reward: 0,
            jurors_reward: 10,
            creators_reward: 10,
            operators_reward: 0,
            cooldown_epochs: 0,
            max_supply: 21_000_000u128 * 100_000_000u128, // satoshi-like base unit
        }
    }
}

#[derive(Default)]
struct Ledger {
    balances: HashMap<String, u128>,
    pool_members: HashMap<String, HashSet<String>>,
    last_win_epoch: HashMap<String, HashMap<String, u64>>,
    eligible: HashSet<String>,
    config: Config,
    current_epoch: u64,
    events: Vec<Event>,
    total_supply: u128,
}

impl Ledger {
    fn new() -> Self {
        Self {
            pool_members: HashMap::new(),
            last_win_epoch: HashMap::new(),
            events: Vec::new(),
            config: Config::default(),
            total_supply: 0,
            ..Default::default()
        }
    }

    fn ensure_account(&mut self, id: &str) {
        self.balances.entry(id.to_string()).or_insert(0);
    }

    fn deposit_internal(&mut self, id: &str, amt: u128) -> bool {
        self.ensure_account(id);
        let new_supply = self.total_supply.saturating_add(amt);
        if new_supply > self.config.max_supply {
            return false;
        }
        let e = self.balances.entry(id.to_string()).or_insert(0);
        *e = e.saturating_add(amt);
        self.total_supply = new_supply;
        true
    }

    fn deposit_no_supply(&mut self, id: &str, amt: u128) {
        // use this when transferring existing supply (internal)
        self.ensure_account(id);
        let e = self.balances.entry(id.to_string()).or_insert(0);
        *e = e.saturating_add(amt);
    }

    fn withdraw(&mut self, id: &str, amt: u128) -> bool {
        let e = self.balances.entry(id.to_string()).or_insert(0);
        if *e >= amt {
            *e -= amt;
            true
        } else {
            false
        }
    }

    fn reward_for(&self, pool: &str) -> u128 {
        match pool {
            POOL_TREASURY => self.config.treasury_reward,
            POOL_JURORS => self.config.jurors_reward,
            POOL_CREATORS => self.config.creators_reward,
            POOL_OPERATORS => self.config.operators_reward,
            _ => 0,
        }
    }
}

// PyO3 wrapper
#[pyclass]
pub struct WeCoinLedger {
    inner: Mutex<Ledger>,
}

#[pymethods]
impl WeCoinLedger {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Ledger::new()),
        }
    }

    // --- Accounts ---
    pub fn create_account(&self, id: &str) {
        let mut l = self.inner.lock();
        l.ensure_account(id);
    }

    pub fn balance(&self, id: &str) -> u128 {
        let l = self.inner.lock();
        *l.balances.get(id).unwrap_or(&0)
    }

    pub fn deposit(&self, id: &str, amount: u128) -> PyResult<bool> {
        let mut l = self.inner.lock();
        // deposit mints new tokens and respects max_supply
        Ok(l.deposit_internal(id, amount))
    }

    pub fn transfer(&self, from: &str, to: &str, amount: u128) -> PyResult<bool> {
        let mut l = self.inner.lock();
        if l.withdraw(from, amount) {
            l.deposit_no_supply(to, amount);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn withdraw(&self, id: &str, amount: u128) -> PyResult<bool> {
        let mut l = self.inner.lock();
        Ok(l.withdraw(id, amount))
    }

    // --- Config ---
    pub fn set_epoch_rewards(&self, treasury: u128, jurors: u128, creators: u128, operators: u128) {
        let mut l = self.inner.lock();
        l.config.treasury_reward = treasury;
        l.config.jurors_reward = jurors;
        l.config.creators_reward = creators;
        l.config.operators_reward = operators;
    }

    pub fn set_cooldown_epochs(&self, cooldown: u64) {
        let mut l = self.inner.lock();
        l.config.cooldown_epochs = cooldown;
    }

    pub fn set_max_supply(&self, max_supply: u128) {
        let mut l = self.inner.lock();
        l.config.max_supply = max_supply;
    }

    pub fn set_epoch(&self, epoch: u64) {
        let mut l = self.inner.lock();
        l.current_epoch = epoch;
    }

    // --- Eligibility / Pools ---
    pub fn set_eligible(&self, account: &str, eligible: bool) {
        let mut l = self.inner.lock();
        if eligible {
            l.eligible.insert(account.to_string());
        } else {
            l.eligible.remove(account);
            // remove from all pools
            for members in l.pool_members.values_mut() {
                members.remove(account);
            }
        }
    }

    pub fn clear_pool(&self, pool: &str) -> PyResult<()> {
        let mut l = self.inner.lock();
        l.pool_members.insert(pool.to_string(), HashSet::new());
        Ok(())
    }

    pub fn add_to_pool(&self, pool: &str, account: &str) -> PyResult<()> {
        let mut l = self.inner.lock();
        l.ensure_account(account);
        // guard for operators
        if pool == POOL_OPERATORS && !l.eligible.contains(account) {
            return Err(pyo3::exceptions::PyPermissionError::new_err("operator not eligible"));
        }
        l.pool_members.entry(pool.to_string()).or_insert_with(HashSet::new).insert(account.to_string());
        Ok(())
    }

    pub fn list_pool_members(&self, pool: &str) -> PyResult<Vec<String>> {
        let l = self.inner.lock();
        match l.pool_members.get(pool) {
            Some(set) => Ok(set.iter().cloned().collect()),
            None => Ok(vec![]),
        }
    }

    pub fn list_all_pools(&self) -> PyResult<Vec<(String, Vec<String>)>> {
        let l = self.inner.lock();
        let mut out = Vec::new();
        for (k, set) in l.pool_members.iter() {
            out.push((k.clone(), set.iter().cloned().collect()));
        }
        Ok(out)
    }

    // --- Epoch control with cooldown and reward supply check ---
    pub fn distribute_epoch_rewards(&self, seed: u64) -> PyResult<HashMap<String, Option<String>>> {
        let mut l = self.inner.lock();
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut winners: HashMap<String, Option<String>> = HashMap::new();

        let pools = vec![POOL_TREASURY, POOL_JURORS, POOL_CREATORS, POOL_OPERATORS];

        for pool in pools {
            let key = pool.to_string();
            let members: Vec<String> = l.pool_members.get(pool).map(|s| s.iter().cloned().collect()).unwrap_or_default();

            if members.is_empty() {
                winners.insert(key.clone(), None);
                continue;
            }

            let cooldown = l.config.cooldown_epochs;
            let current_epoch = l.current_epoch;
            let wins_map = l.last_win_epoch.get(pool).cloned().unwrap_or_default();

            let eligible_now: Vec<String> = members.into_iter().filter(|a| {
                wins_map.get(a).map_or(true, |last| current_epoch.saturating_sub(*last) >= cooldown)
            }).collect();

            if eligible_now.is_empty() {
                winners.insert(key.clone(), None);
                continue;
            }

            let idx = rng.gen_range(0..eligible_now.len());
            let winner = eligible_now[idx].clone();
            let reward = l.reward_for(pool);

            if reward > 0 {
                // attempt to mint reward; respects max_supply
                let ok = l.deposit_internal(&winner, reward);
                if !ok {
                    // couldn't mint due to cap; leave winner None and continue
                    winners.insert(key.clone(), None);
                    continue;
                }
            }

            l.last_win_epoch.entry(pool.to_string()).or_insert_with(HashMap::new).insert(winner.clone(), current_epoch);
            winners.insert(key, Some(winner));
        }

        Ok(winners)
    }

    // --- Slashing ---
    pub fn slash(&self, account: &str, amount: u128, to_pool: &str) -> PyResult<bool> {
        let mut l = self.inner.lock();
        let ok = l.withdraw(account, amount);
        if ok {
            // deposit to a pool account (internal accounting)
            let pool_key = format!("pool:{}", to_pool);
            l.deposit_no_supply(&pool_key, amount);
        }
        Ok(ok)
    }

    // --- Events (structured JSON) ---
    pub fn add_event(&self, tag: &str, details: &str) -> PyResult<()> {
        let mut l = self.inner.lock();
        let parsed: serde_json::Value = serde_json::from_str(details)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
        let evt = Event {
            tag: tag.to_string(),
            details: parsed,
            timestamp: Utc::now().timestamp(),
        };
        l.events.push(evt);
        Ok(())
    }

    pub fn list_events(&self, count: Option<usize>) -> PyResult<String> {
        let l = self.inner.lock();
        let total = l.events.len();
        let slice = match count {
            Some(c) if c < total => l.events[total - c..].to_vec(),
            _ => l.events.clone(),
        };
        let s = serde_json::to_string(&slice).map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Serialize error: {}", e)))?;
        Ok(s)
    }

    // helper: get total_supply
    pub fn total_supply(&self) -> PyResult<u128> {
        let l = self.inner.lock();
        Ok(l.total_supply)
    }
}

#[pymodule]
fn wecoin(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<WeCoinLedger>()?;
    Ok(())
}
