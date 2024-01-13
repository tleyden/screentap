
Screentap lets you tap into the rich activity happening on your desktop screen.  

If you know you looked at something but forgot where you saw it (was it web browsing, email, twitter?), you can pull up screentap and quickly search for it.

![Screenshot](https://github.com/tleyden/screentap/assets/296876/bd865946-68fb-4ff5-8982-024cc4d5bce0 | width=400)

You can also "replay" your screenshots, essentially watching a movie of what you were doing.

**The Vision**

The "tap" of your stream of screen activity is a very rich source of data, especially when you further process it by OCR and newer multimodal vision models such as [Llava](https://llava-vl.github.io/).  

The vision for screentap is to move beyond just searching and browsing your screenshot history to build an extensible platform with plugins for various specialized use cases:

* **Billable hours tracking** - for freelancers and indie hackers juggling multiple projects
* **Habit/behavior tracking and alerting** - get alerged when spending too much time on addictive sites such as X.com, while giving yourself a pass for content that is relevant for your work or hobbies
* **Efficiency suggestions** - by watching your screen usage, a plugin could spot inefficiences in your workflows and suggest improvements 
* **ADHD metrics** - constantly switching contexts?  A plugin could help put a number on that

# Current status

This app is still pre-alpha.  Here's what you can currently do it with it:

1. Run it in the background to periodically capture and OCR full-screen screenshots every 60s
2. Search screenshots by keyword (25 results max)
3. Browse the most recently cpatured screenshot (soon it will let you scroll through a timeline)

# How it works

Screentap runs in the background and periodically takes screenshots of your screen.  It runs each image through the [Apple VisionKit API](https://developer.apple.com/documentation/visionkit) to get the text in the image via OCR.

The images and OCR text are stored in a sqlite database, which can then be searched and browsed from the UI:

# Security and privacy

The screenshots and OCR'd text never leave your computer.

# Install dependencies

## Install rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Install Tauri

[Tauri](http://tauri.app) is a modern framework for building native apps using web technology. 

```
cargo install create-tauri-app --locked
```

# Run screentap

```
cd screentap-app
yarn tauri dev
```


# Inspiring projects

## Open Source

* [rem](https://github.com/jasonjmcghee/rem)
* [tutt](https://github.com/tleyden/tutt)

## Proprietary

* [rewind.ai](https://rewind.ai)