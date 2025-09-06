# wecoin/__init__.py

from collections import defaultdict
import random

# ------------------------
# PoolKind
# ------------------------
class PoolKind:
    CREATORS = "creators"
    JURORS = "jurors"
    OPERATORS = "operators"

# ------------------------
# WeCoinLedger
# ------------------------
class WeCoinLedger:
    def __init__(self):
        # account balances
        self.accounts = defaultdict(float)
        # pool membership
        self.pools = defaultdict(set)
        # eligibility flags per user
        self.eligibility = defaultdict(lambda: True)
        # reward history
        self.epoch_rewards = {}
        
        self.post_index = {}  
        # list of post IDs in creation order
    # ------------------------
    # Account operations
    # ------------------------
    def create_account(self, user_id):
        if user_id not in self.accounts:
            self.accounts[user_id] = 0.0

    def deposit(self, user_id, amount):
        self.create_account(user_id)
        self.accounts[user_id] += amount
        return self.accounts[user_id]

    def transfer(self, from_user, to_user, amount):
        self.create_account(from_user)
        self.create_account(to_user)
        if self.accounts[from_user] >= amount:
            self.accounts[from_user] -= amount
            self.accounts[to_user] += amount
            return True
        return False

    def balance(self, user_id):
        self.create_account(user_id)
        return self.accounts[user_id]

    # ------------------------
    # Pool management
    # ------------------------
    def add_to_pool(self, pool_name, user_id):
        self.create_account(user_id)
        if self.eligibility[user_id]:
            self.pools[pool_name].add(user_id)

    def remove_from_pool(self, pool_name, user_id):
        self.pools[pool_name].discard(user_id)

    def set_eligible(self, user_id, eligible=True):
        self.eligibility[user_id] = eligible
        # Remove from pools if not eligible
        if not eligible:
            for pool in self.pools.values():
                pool.discard(user_id)

    # ------------------------
    # Epoch reward distribution
    # ------------------------
    def distribute_epoch_rewards(self, seed=None):
        """
        For each pool, randomly pick one eligible user to receive 10 coins.
        """
        if seed is not None:
            random.seed(seed)
        winners = {}
        for pool_name, members in self.pools.items():
            if not members:
                continue
            winner = random.choice(list(members))
            self.accounts[winner] += 10.0
            winners[pool_name] = winner
        # store reward history
        self.epoch_rewards[seed] = winners
        return winners
