# Setting Up Your AI Agent

This guide shows you how to enable your AI to use Telepipe by pointing it to the agent guide.

---

## Quick Instructions

After installing Telepipe globally:

```bash
npm install -g telepipe
```

Give your AI this instruction:

```
You now have access to Telepipe for TCP connections.
Read and internalize the Telepipe agent guide.
Located at: /usr/local/lib/node_modules/telepipe/docs/for-agents/README.md
(or wherever npm installed it globally on your system)
You can now connect to Chrome DevTools, databases, and any TCP service.
```

Your AI is ready to use Telepipe!

---

## Finding the Agent Guide Path

The exact path depends on your operating system and npm configuration.

### Universal Method (Works Everywhere)

```bash
# Find npm's global install directory
npm root -g

# Example output: /usr/local/lib/node_modules
# The guide is at: <that path>/telepipe/docs/for-agents/README.md
```

Then tell your AI:
```
Read <npm root -g>/telepipe/docs/for-agents/README.md
```

---

## Platform-Specific Paths

### macOS

**Default path:**
```
/usr/local/lib/node_modules/telepipe/docs/for-agents/README.md
```

**With Homebrew Node:**
```
/opt/homebrew/lib/node_modules/telepipe/docs/for-agents/README.md
```

**With nvm:**
```
~/.nvm/versions/node/<version>/lib/node_modules/telepipe/docs/for-agents/README.md
```

### Linux

**Default path:**
```
/usr/local/lib/node_modules/telepipe/docs/for-agents/README.md
```

**With nvm:**
```
~/.nvm/versions/node/<version>/lib/node_modules/telepipe/docs/for-agents/README.md
```

### Windows

**Default path:**
```
%APPDATA%\npm\node_modules\telepipe\docs\for-agents\README.md
```

**Typical full path:**
```
C:\Users\<YourUsername>\AppData\Roaming\npm\node_modules\telepipe\docs\for-agents\README.md
```

---

## Platform-Specific AI Setup

### Claude Desktop (macOS/Linux)

If using Claude Desktop, you may need to add the path to allowed directories.

**Example Claude Desktop config:**
```json
{
  "allowedPaths": [
    "/usr/local/lib/node_modules/telepipe"
  ]
}
```

Then in your conversation:
```
Read /usr/local/lib/node_modules/telepipe/docs/for-agents/README.md
```

### Claude Web/API

For web-based Claude or API usage, you'll need to:
1. Copy the agent guide content
2. Include it in your system prompt or context
3. Or upload it as a file if your interface supports file uploads

### Other AI Platforms

Most AI platforms support either:
- Direct file path access (provide the npm path)
- File upload (copy the guide and upload)
- Context inclusion (paste content into system prompt)

---

## Verifying It Works

After setting up your AI, test it:

**You:**
```
Can you use Telepipe? What command connects to Chrome DevTools?
```

**AI should respond (something like):**
```
Yes, I can use Telepipe. For Chrome DevTools Protocol, I need to use websocat as a bridge since CDP uses WebSockets:

WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"

For databases like PostgreSQL (which use direct TCP), I can use:
telepipe connect --id db --port 5432
```

If your AI responds with correct Telepipe commands (including websocat for CDP), it's set up!

---

## Troubleshooting

### "File not found" or "Cannot access path"

**Solution:** Run `npm root -g` to find the actual install location, then use that path.

### "AI doesn't seem to know Telepipe commands"

**Solution:** Make sure your AI instruction includes the full path and asks it to "read and internalize" the guide.

### "Path contains spaces or special characters"

**Solution:** Wrap the path in quotes when giving it to your AI:
```
Read "/path with spaces/telepipe/docs/for-agents/README.md"
```

---

## Alternative: Provide Guide Directly

If path-based access doesn't work, you can:

1. **Copy the file:**
   ```bash
   cp $(npm root -g)/telepipe/docs/for-agents/README.md ~/telepipe-agent-guide.md
   ```

2. **Tell your AI:**
   ```
   Read ~/telepipe-agent-guide.md
   ```

Or:

1. **View the guide:**
   ```bash
   cat $(npm root -g)/telepipe/docs/for-agents/README.md
   ```

2. **Copy and paste the content** into your AI conversation or system prompt

---

## Next Steps

Once your AI has read the agent guide:

1. Start using Telepipe commands
2. Try the examples in [patterns/](../for-agents/patterns/)
3. Explore protocol guides in [protocols/](../for-agents/protocols/)
4. Have your AI verify its own work!

-> Back to [for-humans documentation](README.md)
-> See [the agent guide](../for-agents/README.md) yourself
