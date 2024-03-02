
Screentap lets you tap into the rich activity happening on your desktop screen.  

Suppose you saw something but can't even remember where you saw it - was it web browsing, email, twitter?  It's hard to find something when you don't even know where to start searching.  

Screentap gives you a universal search interface to perform keyword searches across anything that has appeared on your screen.

<div align="center">
    <img width="800" alt="Screenshot" src="https://github.com/tleyden/screentap/assets/296876/bd865946-68fb-4ff5-8982-024cc4d5bce0">
</div>

You can also browse your screenshot history to see what you were doing on your computer during a given time period.

**Plugins**

The vision for screentap is to move beyond just searching and browsing your screenshot history, and provide an extensible plugin platform.

There is currently a single plugin available:

* **FocusGuard distraction alerting** - see the [FocusGuard README](screentap-app/plugins/focusguard/) for details and installation quickstart.

The "tap" of your stream of screen activity is a very rich data source, especially when you further process it by OCR and newer multimodal AI vision models.  Some ideas for other plugins:

* **Efficiency suggestions** - Spot inefficiences in your workflows and get suggestions for improvements 
* **Billable hours tracking** - Freelancers and indie hackers that are juggling multiple projects could track billable hours

If you have an idea for a plugin you would want, feel free to [file an issue](https://github.com/tleyden/screentap/issues)!

# Current status + limitations

This app is still pre-alpha.  Here's what you can currently do it with it:

1. Run it in the background to periodically capture and OCR full-screen screenshots every 60s
2. Search screenshots by keyword (25 results max)
3. Browse the most recently captured screenshot (soon it will let you scroll through a timeline)
4. Enable [FocusGuard](screentap-app/plugins/focusguard/) distraction alerts

See the [issue list](https://github.com/tleyden/screentap/issues) for planned improvements. 

# How it works

Screentap is a native OSX app that runs in the background and periodically takes screenshots of your screen.  It processes each captured image through the [Apple VisionKit API](https://developer.apple.com/documentation/visionkit) to get the text in the image via OCR.

The images and OCR text are stored in a sqlite database, which can then be searched and browsed from the UI:

# Security and privacy

The screenshots and OCR text never leave your computer.  As an open source project, the screentap code and 3rd party libraries are available to audit so you can verify this is the case. 

To delete your history screenshot, navigate to `/Users/<username>/Library/Application Support/com.screentap-app.dev` in the OSX terminal and remove all files in that directory.

## Security risks

Screenshots may contain secrets.  If leaked, this could allow an attacker to infiltrate your other logins.  There is a [task](issue) to add a retention policy to minimize the chance of this happening.

# Running screentap

There are no compiled binaries available yet, so to run it you will need to clone the repo, install [Tauri](http://tauri.app), and build/run the native Tauri app.

## Install dependencies

### Install rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install Tauri framework

[Tauri](http://tauri.app) is a modern framework for building native apps using web technology. 

```
cargo install create-tauri-app --locked
```

Make sure you end up with the `tauri` binary on your system, you might have to do something like this:

```
cargo install tauri-cli
ln -s ~/.cargo/bin/cargo-tauri ~/.cargo/bin/tauri
```

## Build and run screentap

```
cd screentap-app
yarn install vite
yarn tauri dev
```


# Projects that inspired screentap

## Open Source

* [rem](https://github.com/jasonjmcghee/rem)
* [tutt](https://github.com/tleyden/tutt)

## Proprietary

* [rewind.ai](https://rewind.ai)
* [rize.io](https://rize.io)
