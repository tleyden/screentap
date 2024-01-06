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

(I think yarn gets installed by `cargo install create-tauri-app`)

## Run Tauri app

```
cd screentap-app
yarn tauri dev
```