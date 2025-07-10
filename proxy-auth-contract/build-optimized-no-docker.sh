RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --lib
wasm-opt -Oz target/wasm32-unknown-unknown/release/autobidder.wasm -o target/wasm32-unknown-unknown/release/autobidder_opt.wasm
