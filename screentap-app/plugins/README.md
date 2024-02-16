
The content of this dir should somehow get deployed into the application root:

```
 /Users/<username>/Library/Application Support/com.screentap-app.dev
```

 eg, 

```
 ../com.screentap-app.dev/plugins/focusguard/config.toml
```

This is where the application will expect to find these files.

Create a symlink as follows:

```
ln -s "/Users/<username>/Development/screentap/screentap-app/plugins/"  "/Users/tleyden/Library/Application Support/com.screentap-app.dev/"
```