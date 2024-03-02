FocusGuard is an OSX desktop app that helps you stay focused by analyzing your screen with AI vision models.  

You can choose between two vision models:

1. OpenAI [GPT-Vision](https://platform.openai.com/docs/guides/vision) 
2. [Llava1.5](https://llava-vl.github.io/) open source model

FocusGuard runs in the background and analyzes your screen approximately every 30s.  When it detects you aren't working it will pop up a screen like this:

<img src="https://github.com/tleyden/screentap/assets/296876/44a49ed2-84a2-46d7-bad9-b898571c848a" height="300">

Expanding the "Details" section will show the screenshot that triggered the alert, along with an explanation from the LLM as to why it thought you were getting distracted.

<img src="https://github.com/tleyden/screentap/assets/296876/25946863-e104-4dd9-835e-fc5cecdaee70" height="300">


## System requirements

1. If you configure it to use OpenAI [GPT-Vision](https://platform.openai.com/docs/guides/vision) you will need a paid [ChatGPT Plus](https://openai.com/blog/chatgpt-plus) subscription.
2. Otherwise if you configure it to use Llava1.5 open source model you will need an M1 or later Mac with 5GB of free space.  See [llamafile](https://github.com/Mozilla-Ocho/llamafile) for detailed requirements.  This is of course free to run.

## Quick start

### Step 1: Download and run screentap

Download [screentap](https://github.com/tleyden/screentap) from a pre-built binary (not available at time of writing) or by cloning the repo and building it locally.

Run screentap and make sure its working (see [screentap README](https://github.com/tleyden/screentap))

### Step 2: Create FocusGuard configuration 

Download the [FocusGuard config sample](screentap-app/plugins/focusguard/config_sample.toml) to your computer and save it to this location:

```
/Users/<your username>/Library/Application Support/com.screentap-app.prod/plugins/focusguard/config.toml
```

If the directory doesn't exist, make sure you have run screentap at least once.  If it still doesn't exist, [report a bug here](https://github.com/tleyden/screentap/issues).

Customize the **job_title** and **job_role** configuration fields for your use case, otherwise you won't get decent results.

If you leave the `llava_backend` as `LlamaFileSubprocess`, there is nothing more to do.  It will download the model on the first use, and there is no API key needed since it is an open source local model.

OTOH of you set the  `llava_backend` to be `OpenAI`, you will also need to set your openai key in the `open_ai_api_key`.  This isn't ideal from a security point of view, and hopefully in the future this will be improved to store this in the Apple Keychain instead.

### Step 3: Restart screentap

Restart screentap to activate focusguard.

### Verify installation

If you are running the precompiled binary from the DMG file (as opposed to compiling it and running it from the CLI), there are unfortunately no logs to look at to verify that FocusGuard correctly loaded.

Your best bet is to use it for a few hours and purposely look at distracting content irrelevant to your job, then wait for distraction alerts.  There is intentionally a few minutes delay to reduce the noise from distraction alerts, so you will need to linger on distracting content for a few minutes to trigger it.

If you are running from the CLI, you will be able to see the raw output when it invokes the AI vision model.  On startup, you should see messages like this on the terminal.

<details open="false">
<summary>Example logs</summary>

```
FocusGuard config found at path: /Users/<your username>/Library/Application Support/com.screentap-app.dev/plugins/focusguard/config.toml
Capturing screenshot.  cur_frontmost_app: missing value last_frontmost_app: com.googlecode.iterm2 cur_browser_tab: , last_browser_tab:  frontmost_app_or_tab_changed: true
FocusGuard handling screentap event # 7849 with len(ocr_text): 139 and len(png_data): 494254 frontmost app: missing value frontmost browser tab:
```

</details>

and when it invokes the AI vision model, you should see messages like this on the terminal:

<details open="false">
<summary>Example logs</summary>

```
FocusGuard analyzing image with OpenAI.  Resizing image at png_image_path: ..
Resized image length in bytes: 548430: time_to_resize: 14.5264895s
Invoking OpenAI API
time_to_infer: 10.707368s
```

</details>

## Status

The app is "pre-alpha" and is targeted towards folks that are interested and technical enough to deal with the lack of polish.  For example, currently you need to configure it with a text editor rather than the UI.

Only use this app if you understand the security risk of saving automatically collected screenshots to your system.  They may contain highly sensitive information such as passwords, personal financial details, etc.  They will NOT be transmitted, however just the act of saving them in an unencrypted format carries additional risks.

## License

Apache 2 (same as screentap)

## How it works under the hood

FocusGuard is designed as a [screentap](https://github.com/tleyden/screentap) plugin that is optionally enabled if a configuration file is present. 

Screentap captures full-screen screenshots every 30s, saves them to disk for searching/browsing, and then invokes FocusGuard for further analysis.

FocusGuard runs custom logic to see if the screenshot should even be analyzed by the AI Vision model, since this can be computationally or financially expensive.  The logic can be summarized as follows:

1. If the user has switched screens or browser tabs in the last 90 seconds, then assume the user is transitioning between apps and ignore the screenshot.  This avoids spurious alerts.
2. If the user is using Screentap itself, ignore the screenshot
3. Otherwise, analyze the screenshot with the AI Vision model.

The distraction alert currently has a "yes/no" feedback mechanism, but those responses are currently ignored.  Hopefully in the future they will be recorded and used as a feedback loop to improve the performance.

