## One time setup

Install rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install Tauri

```
cargo install create-tauri-app --locked
```

Install Yarn

(I think this gets insalled by `cargo install create-tauri-app`)

## Running Tauri app

```
cd screentap-app
yarn tauri dev
```