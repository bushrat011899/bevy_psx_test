# Bevy on the PSX

This is not meant to be.
Wanna know how not-meant-to-be it is?

## Patching LLVM

```sh
git clone https://github.com/rust-lang/rust.git
cd rust
git checkout a4cb3c831823d9baa56c3d90514b75b2660116fa

cp config.example.toml config.toml
# Set lld to true
sed -i 's/#lld = false/lld = true/' config.toml
# Only build the MIPS and X86 targets
sed -i 's/#targets.*$/targets = "Mips;X86"/' config.toml
# Don't build any experimental targets
sed -i 's/#experimental-targets.*$/experimental-targets = ""/' config.toml

git submodule update --init --progress src/llvm-project
cd src/llvm-project
git apply ../../../llvm_atomic_fence.patch

cd ../..

python ./x.py build -i library/std
rustup toolchain link psx build/x86_64-pc-windows-msvc/stage1
```
