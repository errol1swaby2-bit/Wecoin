**v0.6 is up with the wecoin mvp**
**they are tagged into this repo**
**at the current moment i dont** **know how to push it into main.**
# WeCoin

WeCoin is a lightweight, Python-Rust hybrid cryptocurrency ledger designed for integration with decentralized social protocols. It provides account management, pool-based reward distribution, and eligibility tracking for creators, jurors, and operators.

---

## **Features**

- **Account Management**: Create accounts, deposit, transfer, and check balances.
- **Pool System**: Users can be added to pools (Creators, Jurors, Operators) for reward distribution.
- **Eligibility Control**: Mark users eligible or ineligible for rewards.
- **Epoch-Based Rewards**: Randomized reward distribution per pool with cooldown support.
- **Slashing Mechanism**: Penalize misbehaving accounts by redistributing their funds to pools.
- **Python-Rust Integration**: High-performance Rust backend exposed via PyO3 Python bindings.

---

## **Installation**

**Requirements:**

- Python 3.10+
- Rust 1.70+
- PyO3, rand, rand_chacha, serde, serde_json (included in Cargo.toml)

**Install from source:**

```bash
# Clone the repo
git clone git@github.com:errol1swaby2-bit/Wecoin.git
cd Wecoin

# Build the Rust Python module
maturin develop  # Or `python3 -m pip install .` if using setup.py

# Test import
python -c "import wecoin; ledger = wecoin.WeCoinLedger(); print(ledger.balance('Alice'))"
