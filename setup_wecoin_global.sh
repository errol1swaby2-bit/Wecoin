#!/data/data/com.termux/files/usr/bin/bash
# Termux Rust + Python + Maturin + WeCoin setup (global install)

set -e

echo "[*] Updating Termux packages..."
pkg update -y
pkg upgrade -y

echo "[*] Installing build essentials..."
pkg install -y rust python python-pip clang make cmake pkg-config openssl git wget unzip

echo "[*] Removing rustup if present..."
if [ -f "$HOME/.cargo/bin/rustup" ]; then
    rm -f ~/.cargo/bin/rustup
    rm -rf ~/.rustup
    rm -rf ~/.cargo/bin
fi

export PATH="$HOME/.cargo/bin:$PATH"

echo "[*] Verifying Rust installation..."
rustc --version
cargo --version

echo "[*] Upgrading Python packaging tools..."
pip install --upgrade pip setuptools wheel

echo "[*] Installing maturin..."
pip install maturin

echo "[*] Building and installing wecoin globally..."
cd ~/wecoin || { echo "[!] ~/wecoin not found"; exit 1; }

# Build a wheel and install it globally
maturin build --release
pip install target/wheels/wecoin-*.whl

echo "[*] Testing wecoin installation..."
python3 - << EOF
try:
    import wecoin
    print("[✓] wecoin imported successfully!")
except Exception as e:
    print("[✗] Failed to import wecoin:", e)
EOF

echo "[*] Setup complete! WeCoin is ready to use globally in Python."
