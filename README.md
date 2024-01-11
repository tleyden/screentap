
An open-source privacy first clone of the popular Rewind.ai commercial app.  Heavily inspired by [rem](https://github.com/jasonjmcghee/rem).

![Screenshot](https://github.com/tleyden/screentap/assets/296876/e8e48f8c-3aaf-478d-bdb5-f64cc3e27da1)

How it works:

It runs in the background and takes a screenshot every 2 minutes.  It runs each image through the OSX VisionKit to get the text in the image via OCR.

All images and OCR'd text are stored in a sqlite database, which can then be searched.

The app is a hybrid native/web app built on the [Tauri](http://tauri.app) framework. 

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