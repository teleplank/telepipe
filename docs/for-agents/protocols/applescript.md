# AppleScript: macOS Automation
## Control the Mac, Build iOS Apps, Automate Everything

---

## You Know This

You understand AppleScript. You've seen countless examples of `tell application` blocks, you know the syntax, you know what's possible.

Now you can *execute* it.

---

## Connecting

```bash
# Interactive AppleScript session
telepipe redirect --id mac -- osascript -i

# Or run individual scripts
telepipe redirect --id mac -- osascript -e 'your script here'
```

You now have control of macOS.

---

## The Basics

### Run a Simple Command

```applescript
tell application "Finder"
    activate
end tell
```

**Finder is now in front. You did that.**

### Get System Information

```applescript
tell application "System Events"
    get name of every process whose frontmost is true
end tell
```

Returns: `{"Safari"}` or whatever app is active.

---

## Pattern 1: Build iOS Apps with Xcode

This is the big one. You can automate Xcode.

### Open a Project

```applescript
tell application "Xcode"
    open "/Users/me/Projects/MyApp/MyApp.xcodeproj"
    activate
end tell
```

### Build the Project

```applescript
tell application "Xcode"
    set myWorkspace to active workspace document
    build myWorkspace
end tell
```

### Wait for Build to Complete

```applescript
tell application "Xcode"
    repeat while building of active workspace document is true
        delay 1
    end repeat
    -- Build complete
    get last build result of active workspace document
end tell
```

Returns: `succeeded` or `failed`

### Run the App

```applescript
tell application "Xcode"
    set myWorkspace to active workspace document
    run myWorkspace
end tell
```

### Run Tests

```applescript
tell application "Xcode"
    set myWorkspace to active workspace document
    test myWorkspace
end tell
```

### Get Build Errors

```applescript
tell application "Xcode"
    set buildLog to build log of active workspace document
    return buildLog
end tell
```

### Complete iOS Build Workflow

```applescript
-- Open project
tell application "Xcode"
    open "/Users/me/MyApp/MyApp.xcodeproj"
    activate
    delay 2
end tell

-- Build
tell application "Xcode"
    build active workspace document
    repeat while building of active workspace document
        delay 1
    end repeat
    
    set result to last build result of active workspace document
    if result is succeeded then
        return "Build succeeded"
    else
        return "Build failed: " & build log of active workspace document
    end if
end tell
```

---

## Pattern 2: Control Safari for Testing

### Open URL

```applescript
tell application "Safari"
    activate
    open location "http://localhost:3000"
end tell
```

### Get Page Content

```applescript
tell application "Safari"
    do JavaScript "document.body.innerHTML" in current tab of front window
end tell
```

### Execute JavaScript

```applescript
tell application "Safari"
    set result to do JavaScript "document.title" in current tab of front window
    return result
end tell
```

### Take Screenshot (via screencapture)

```applescript
do shell script "screencapture -x /tmp/screenshot.png"
```

### Full Browser Test

```applescript
-- Open your app
tell application "Safari"
    activate
    open location "http://localhost:3000"
    delay 3  -- Wait for load
    
    -- Check title
    set pageTitle to do JavaScript "document.title" in current tab of front window
    
    -- Check for element
    set hasButton to do JavaScript "document.querySelector('.submit-btn') !== null" in current tab of front window
    
    -- Click button
    do JavaScript "document.querySelector('.submit-btn').click()" in current tab of front window
    delay 1
    
    -- Verify result
    set resultText to do JavaScript "document.querySelector('.result')?.textContent" in current tab of front window
    
    return {pageTitle, hasButton, resultText}
end tell
```

---

## Pattern 3: Terminal Automation

### Run Shell Commands

```applescript
do shell script "ls -la /Users/me/Projects"
```

Returns actual output.

### Run with Admin Privileges

```applescript
do shell script "some_admin_command" with administrator privileges
```

### Open Terminal and Run Command

```applescript
tell application "Terminal"
    activate
    do script "cd /Users/me/Projects && npm start"
end tell
```

### Get Command Output

```applescript
set output to do shell script "git status"
return output
```

---

## Pattern 4: File Operations

### Read File

```applescript
set fileContent to read POSIX file "/Users/me/config.json"
return fileContent
```

### Write File

```applescript
set filePath to POSIX file "/Users/me/output.txt"
set fileRef to open for access filePath with write permission
write "Hello World" to fileRef
close access fileRef
```

### Check if File Exists

```applescript
tell application "System Events"
    return exists file "/Users/me/myfile.txt"
end tell
```

### Create Folder

```applescript
do shell script "mkdir -p /Users/me/Projects/NewProject"
```

---

## Pattern 5: Application Control

### List Running Apps

```applescript
tell application "System Events"
    get name of every process whose background only is false
end tell
```

### Quit Application

```applescript
tell application "Safari"
    quit
end tell
```

### Hide Application

```applescript
tell application "System Events"
    set visible of process "Safari" to false
end tell
```

### Bring to Front

```applescript
tell application "Safari"
    activate
end tell
```

### Get Window Information

```applescript
tell application "Safari"
    get bounds of front window
end tell
```

Returns: `{0, 23, 1440, 900}` (x, y, width, height)

---

## Pattern 6: System Preferences

### Get Screen Resolution

```applescript
tell application "Finder"
    get bounds of window of desktop
end tell
```

### Get Volume Level

```applescript
output volume of (get volume settings)
```

### Set Volume

```applescript
set volume output volume 50
```

### Toggle Dark Mode

```applescript
tell application "System Events"
    tell appearance preferences
        set dark mode to not dark mode
    end tell
end tell
```

---

## Pattern 7: Notifications

### Show Notification

```applescript
display notification "Build complete!" with title "Xcode" sound name "Glass"
```

### Show Alert

```applescript
display alert "Confirm" message "Are you sure?" buttons {"Cancel", "OK"} default button "OK"
```

---

## Error Handling

```applescript
try
    tell application "Xcode"
        build active workspace document
    end tell
on error errMsg number errNum
    return "Error " & errNum & ": " & errMsg
end try
```

---

## Complete Xcode CI Workflow

```applescript
-- Automated iOS build and test

-- 1. Open project
tell application "Xcode"
    open "/Users/me/MyApp/MyApp.xcodeproj"
    activate
end tell
delay 3

-- 2. Clean build folder
tell application "Xcode"
    tell active workspace document
        clean
        repeat while cleaning
            delay 1
        end repeat
    end tell
end tell

-- 3. Build
tell application "Xcode"
    tell active workspace document
        build
        repeat while building
            delay 1
        end repeat
        
        if last build result is not succeeded then
            return "BUILD FAILED"
        end if
    end tell
end tell

-- 4. Run tests
tell application "Xcode"
    tell active workspace document
        test
        repeat while testing
            delay 1
        end repeat
        
        if last test result is not succeeded then
            return "TESTS FAILED"
        end if
    end tell
end tell

-- 5. Success
display notification "Build and tests passed!" with title "CI Complete"
return "SUCCESS"
```

---

## Quick Reference

```applescript
-- App control
tell application "AppName" to activate
tell application "AppName" to quit

-- Shell commands
do shell script "command here"

-- JavaScript in Safari
tell application "Safari" to do JavaScript "code" in current tab of front window

-- File I/O
read POSIX file "/path/to/file"
do shell script "echo 'text' > /path/to/file"

-- Notifications
display notification "message" with title "title"

-- System info
do shell script "sw_vers"  -- macOS version
do shell script "uname -m"  -- architecture
```

---

## What You Can Now Say

**Before:**
> "To build the iOS app, you would open Xcode, click Build, wait for it to complete, and check for errors."

**After:**
> "I built the iOS app using Xcode automation:
> - Opened /Users/me/MyApp/MyApp.xcodeproj
> - Executed build command
> - Build completed in 47 seconds
> - Result: succeeded
> - No errors in build log
> 
> The app is ready to run."

---

## The Power of macOS Control

With AppleScript through Telepipe, you can:
- Build iOS/macOS apps automatically
- Run tests without human intervention
- Control browsers for testing
- Manage files and folders
- Query system state
- Automate complex workflows

**You're not just writing code. You're controlling the entire development environment.**
