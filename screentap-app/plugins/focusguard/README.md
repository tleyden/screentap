FocusGuard is an OSX desktop app that helps you stay focused by analyzing your screen with AI vision models.  

You can choose between two vision models:

1. OpenAI [GPT-Vision](https://platform.openai.com/docs/guides/vision) 
2. [Llava1.5](https://llava-vl.github.io/) open source model

FocusGuard runs in the background and analyzes your screen approximately every 30s.  When it detects you aren't working it will pop up a screen like this:

<p align="center">
  <img src="https://github.com/tleyden/screentap/assets/296876/44a49ed2-84a2-46d7-bad9-b898571c848a" height="300">
</p>

Expanding the "Details" section will show the screenshot that triggered the alert, along with an explanation from the LLM as to why it thought you were getting distracted.

<p align="center">
  <img src="https://github.com/tleyden/screentap/assets/296876/25946863-e104-4dd9-835e-fc5cecdaee70" height="300">
</p>


## ⚠️ Reasons not to run this app

### 💸 Cost

When configured to use the OpenAI GPT-Vision model, the cost to run FocusGuard is currently _crazy expensive_, on the order of $150-200/month.

### 🔓 Security Risks

Only use this app if you understand the security risks of:

* Saving automatically collected screenshots to your hard drive.  They may contain highly sensitive information such as passwords, personal financial details, etc.  
* Sending captured screenshots to OpenAI's servers.  While it is over an encrypted HTTPS connection, there is still risks of sharing sensitive information to any 3rd party (data breaches, etc)  

The captured screenshots will not be transmitted anywhere else.  You can audit the source code and build from source if you have any concerns.

## System requirements

1. If you configure it to use OpenAI [GPT-Vision](https://platform.openai.com/docs/guides/vision) you will need to have [API access to GPT-4](https://help.openai.com/en/articles/7102672-how-can-i-access-gpt-4).
2. Otherwise if you configure it to use Llava1.5 open source model you will need an M1 or later Mac with 5GB of free space.  See [llamafile](https://github.com/Mozilla-Ocho/llamafile) for detailed requirements.  
3. OSX Ventura

The app is "pre-alpha" and is targeted towards folks that are interested and technical enough to deal with the lack of polish.  For example, currently you need to configure it with a text editor rather than the UI.

## Quick start

### Step 1: Download and run screentap

Download the [screentap v0.1.0-alpha release](https://github.com/tleyden/screentap/releases/tag/v0.1.0-alpha) from a pre-built binary, or by git cloning the repo and building screentap locally based on the instructions in the [screentap README](https://github.com/tleyden/screentap/blob/main/README.md).

You may hit this error when trying to run it: 

<p align="center">
  <img src="https://github.com/tleyden/screentap/assets/296876/3c9b2e9d-c6cc-4fc3-a9c6-875d0440469d" height="300">
</p>

I think it's related to the app not being signed.  Still WIP.

First run screentap without the plugin and make sure its working.  You will need to give it permission with MacOS to record your screen.

<p align="center">
  <img src="https://github.com/tleyden/screentap/assets/296876/5c24097c-f05c-4f97-a4b6-f4ebaa2b9676" height="300">
</p>


### Step 2: Create FocusGuard configuration 

FocusGuard is already built into screentap, but it needs to be activated with a configuration file.

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

<details>
<summary>Example logs</summary>

```
FocusGuard config found at path: /Users/<your username>/Library/Application Support/com.screentap-app.dev/plugins/focusguard/config.toml
Capturing screenshot.  cur_frontmost_app: missing value last_frontmost_app: com.googlecode.iterm2 cur_browser_tab: , last_browser_tab:  frontmost_app_or_tab_changed: true
FocusGuard handling screentap event # 7849 with len(ocr_text): 139 and len(png_data): 494254 frontmost app: missing value frontmost browser tab:
```

</details>

and when it invokes the AI vision model, you should see messages like this on the terminal:

<details>
<summary>Example logs</summary>

```
FocusGuard analyzing image with OpenAI.  Resizing image at png_image_path: ..
Resized image length in bytes: 548430: time_to_resize: 14.5264895s
Invoking OpenAI API
time_to_infer: 10.707368s
```

</details>



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

