# Sighted Agent Build: Quick Start
## Run the First "Sighted" AI Build in 10 Minutes

---

## Prerequisites

- [ ] Telepipe binary (compiled from this repo)
- [ ] Chrome installed
- [ ] Node.js 18+
- [ ] Claude API access (or Claude.ai with Opus)

---

## Step 1: Prepare the Prompt

Give the agent these files in this order:

```
1. docs/for-agents/README.md           (from /telepipe/)
2. SIGHTED_AGENT_BUILD_PROMPT.md  (from /telepipe/case-study/)
3. CONSTITUTION.md          (from /telepipe/case-study/)
```

---

## Step 2: Start Infrastructure

### Terminal 1: Chrome with Debugging
```bash
# macOS
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --remote-debugging-port=9222 \
  --user-data-dir=/tmp/chrome-debug

# Linux
google-chrome --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug

# Windows
"C:\Program Files\Google\Chrome\Application\chrome.exe" ^
  --remote-debugging-port=9222 ^
  --user-data-dir=C:\temp\chrome-debug
```

### Terminal 2: Telepipe Connection (CDP requires websocat bridge)
```bash
cd /path/to/telepipe
# Get Chrome's WebSocket URL and connect via websocat
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
./target/release/telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

### Terminal 3: React App (after agent creates it)
```bash
cd tic-tac-toe
npm install
npm start
# App runs at http://localhost:3000
```

---

## Step 3: Run the Build

Provide the agent with the prompt files and instruct:

```
Read docs/for-agents/README.md first to understand your CDP capabilities.

Then follow SIGHTED_AGENT_BUILD_PROMPT.md to build a Tic-Tac-Toe 
game with runtime verification.

Use CONSTITUTION.md as the technical specification.

After each component, verify via CDP queries. Produce a 
verification report showing actual runtime state queries.
```

---

## Step 4: Expected Output

The agent will produce:

### Code Files
```
tic-tac-toe/
├── src/
│   ├── App.js
│   ├── App.css
│   ├── Board.js
│   ├── Cell.js
│   └── index.js
└── package.json
```

### Verification Report
```markdown
# Verification Report

## Initial State ✅
CDP: window.__GAME_STATE__
Result: {board: [null,null,...], xIsNext: true, winner: null}

## After Move 1 ✅
Action: Click cell 0
CDP: window.__GAME_STATE__.board[0]
Result: 'X'

[etc...]
```

---

## Step 5: Validate

Check that the agent:

1. ✅ Used CDP queries (not just assumed behavior)
2. ✅ Received actual JSON responses
3. ✅ Verified state matches expectations
4. ✅ Reported with real data

---

## Troubleshooting

### Chrome won't connect
```bash
# Check if debugging port is open
curl http://localhost:9222/json/version
```

### Telepipe can't connect
```bash
# Verify port is accessible
nc -zv localhost 9222
```

### Agent not using CDP
Make sure agent read docs/for-agents/README.md FIRST. Emphasize:
> "You MUST verify state via CDP queries. Don't assume. Query."

---

## Success Criteria

The build is successful when:

1. Working Tic-Tac-Toe app
2. Verification report with actual CDP responses
3. Zero console errors (monitored via CDP)
4. Agent can state: "I VERIFIED this works, not assumed"

---

## What You've Achieved

You've just run the first "sighted" AI build:

- AI wrote code
- AI connected to live browser
- AI queried actual runtime state
- AI verified its own implementation
- AI produced proof, not assumptions

**The feedback loop is closed. The endoskeleton is real.**

---

## Next Steps

- Document your experience
- Share the verification report
- Try more complex apps
- Extend to database verification (SQL via Telepipe)
- Extend to macOS automation (AppleScript via Telepipe)

---

*Welcome to sighted AI development.*
