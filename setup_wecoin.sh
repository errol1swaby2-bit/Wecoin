#!/data/data/com.termux/files/usr/bin/bash
# Termux Rust + Python + Maturin + WeCoin setup

# 1️⃣ Update Termux packages
pkg update -y
pkg upgrade -y

# 2️⃣ Install build essentials
pkg install -y rust python python-pip clang make cmake pkg-config openssl git wget unzip

# 3️⃣ Remove rustup if installed (to avoid Android panics)
if [ -f "$HOME/.cargo/bin/rustup" ]; then
    echo "[*] Removing rustup..."
    rm -f ~/.cargo/bin/rustup
    rm -rf ~/.rustup
    rm -rf ~/.cargo/bin
fi

# 4️⃣ Ensure Rust & Cargo are in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# 5️⃣ Verify Rust installation
echo "[*] Rust version:"
rustc --version
echo "[*] Cargo version:"
cargo --version

# 6️⃣ Upgrade Python packaging tools
pip install --upgrade pip setuptools wheel

# 7️⃣ Install maturin
pip install maturin

# 8️⃣ Navigate to your wecoin project
cd ~/wecoin || { echo "[!] ~/wecoin not found"; exit 1; }

# 9️⃣ Build wecoin as Python extension
echo "[*] Building wecoin..."
maturin develop

# 10️⃣ Test Python import
echo "[*] Testing wecoin module..."
python3 - << EOF
try:
    import wecoin
    print("[✓] wecoin imported successfully!")
except Exception as e:
    print("[✗] Failed to import wecoin:", e)
EOF

echo "[*] Setup complete!"
