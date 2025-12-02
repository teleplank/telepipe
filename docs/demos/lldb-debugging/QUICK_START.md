# LLDB Debugging: Quick Start
## Debug iOS Apps with AI in 10 Minutes

---

## Prerequisites

- [ ] Telepipe binary (compiled from this repo)
- [ ] Xcode 15+ installed
- [ ] iOS Simulator available
- [ ] Claude API access (or Claude.ai with Opus)

---

## Step 1: Prepare the Prompt

Give the agent these files in this order:

```
1. docs/for-agents/README.md                (from /telepipe/)
2. LLDB_AGENT_PROMPT.md          (from /telepipe/docs/demos/lldb-debugging/)
3. CONSTITUTION.md               (from /telepipe/docs/demos/lldb-debugging/)
```

---

## Step 2: Build the TodoApp

Have the agent build the TodoApp per CONSTITUTION.md, or use this minimal implementation:

### Create Xcode Project
1. Open Xcode → File → New → Project
2. Choose "App" (iOS)
3. Name it "TodoApp"
4. Interface: SwiftUI, Language: Swift

### Replace ContentView.swift
```swift
import SwiftUI

struct TodoItem: Identifiable, Codable {
    let id = UUID()
    var text: String
    var isCompleted: Bool = false
}

class TodoViewModel: ObservableObject {
    @Published var items: [TodoItem] = []

    func addItem(_ text: String) {
        items.append(TodoItem(text: text))
        // BUG: Missing saveItems() call!
    }

    func toggleItem(_ item: TodoItem) {
        if let index = items.firstIndex(where: { $0.id == item.id }) {
            items[index].isCompleted.toggle()
        }
    }

    func loadItems() {
        // BUG: Not loading from UserDefaults!
        items = []
    }

    // These methods exist but aren't called
    private func saveItems() {
        if let encoded = try? JSONEncoder().encode(items) {
            UserDefaults.standard.set(encoded, forKey: "todos")
        }
    }
}

struct ContentView: View {
    @StateObject private var viewModel = TodoViewModel()
    @State private var newTodoText = ""

    var body: some View {
        NavigationView {
            VStack {
                HStack {
                    TextField("New todo", text: $newTodoText)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                    Button("Add") {
                        if !newTodoText.isEmpty {
                            viewModel.addItem(newTodoText)
                            newTodoText = ""
                        }
                    }
                }
                .padding()

                List(viewModel.items) { item in
                    HStack {
                        Image(systemName: item.isCompleted ? "checkmark.circle.fill" : "circle")
                        Text(item.text)
                            .strikethrough(item.isCompleted)
                    }
                    .onTapGesture {
                        viewModel.toggleItem(item)
                    }
                }
            }
            .navigationTitle("Todos")
        }
        .onAppear {
            viewModel.loadItems()
        }
    }
}
```

### Build and Run
1. Select iPhone Simulator as target
2. Press ⌘R to build and run
3. App should appear in simulator

---

## Step 3: Start Infrastructure

### Terminal 1: Verify App is Running
```bash
# Find the running app's process ID
pgrep -l TodoApp
# Should show: 12345 TodoApp
```

### Terminal 2: Attach LLDB via Telepipe
```bash
cd /path/to/telepipe

# Attach LLDB to running app
./target/release/telepipe redirect --id lldb -- lldb -n TodoApp

# You should see:
# (lldb) process attach --name "TodoApp"
# Process 12345 stopped
# Executable module set to "/path/to/TodoApp.app/TodoApp"
```

### Terminal 3: Send LLDB Commands (optional monitoring)
```bash
# You can send commands via exec
echo "po UIApplication.shared" | ./target/release/telepipe exec --id lldb
```

---

## Step 4: Run the Debugging Session

Provide the agent with the prompt files and instruct:

```
Read docs/for-agents/README.md first to understand your LLDB capabilities.

Then follow LLDB_AGENT_PROMPT.md to debug the TodoApp.

Use CONSTITUTION.md as the technical specification.

Your goal:
1. Add todo items via the UI (or via LLDB expressions)
2. Verify items exist via LLDB: po viewModel.items.count
3. Restart the app (stop and relaunch in simulator)
4. Re-attach LLDB
5. Query items again - discover they're gone!
6. Find and fix the persistence bug
7. Verify fix via LLDB

Produce a debugging report showing actual LLDB output.
```

---

## Step 5: Expected Output

The agent will produce:

### Debugging Session Log
```
# Initial Inspection
(lldb) po viewModel.items.count
0

# After Adding "Buy milk"
(lldb) po viewModel.items.count
1
(lldb) po viewModel.items[0].text
"Buy milk"

# After Adding "Walk dog"
(lldb) po viewModel.items.count
2

# After App Restart
(lldb) po viewModel.items.count
0

# BUG DISCOVERED: Items don't persist!

# After Fix Applied
(lldb) po viewModel.items.count
2

# FIX VERIFIED: Items now persist!
```

### Bug Analysis
```markdown
## Bug Found

**Symptom:** Items disappear after app restart
**Location:** TodoViewModel.swift
**Root Cause:** saveItems() never called after addItem()
**Evidence:**
  - Before restart: po viewModel.items.count → 2
  - After restart: po viewModel.items.count → 0
  - po UserDefaults.standard.object(forKey: "todos") → nil

## Fix Applied

Added saveItems() call to addItem():

func addItem(_ text: String) {
    items.append(TodoItem(text: text))
    saveItems()  // Added this line
}

## Fix Verified

- After restart: po viewModel.items.count → 2 ✅
- po UserDefaults.standard.object(forKey: "todos") → <data> ✅
```

---

## Step 6: Validate

Check that the agent:

1. [ ] Used LLDB queries (not just assumed behavior)
2. [ ] Received actual po/p command output
3. [ ] Found the persistence bug via inspection
4. [ ] Verified fix with actual LLDB queries
5. [ ] Produced debugging report with real output

---

## Troubleshooting

### LLDB won't attach
```bash
# Check if app is running
pgrep -l TodoApp

# If not found, launch app from Xcode first

# Check if another debugger is attached
# (Xcode debugger conflicts with command-line LLDB)
# Detach Xcode debugger: Debug → Detach
```

### "process exited" errors
```bash
# App may have crashed. Relaunch in simulator:
# 1. Stop in Xcode (⌘.)
# 2. Run again (⌘R)
# 3. Re-attach LLDB via Telepipe
```

### Can't find viewModel
```bash
# Make sure you're stopped at a breakpoint or paused
# Set a breakpoint first:
(lldb) breakpoint set -n viewDidLoad
(lldb) continue

# Or pause execution:
(lldb) process interrupt
```

### Agent not using LLDB
Make sure agent read docs/for-agents/README.md FIRST. Emphasize:
> "You MUST debug via LLDB queries. Don't assume. Inspect."

---

## Success Criteria

The debugging session is successful when:

1. Working TodoApp running in simulator
2. Bug discovered via LLDB inspection (not guessing)
3. Fix verified via LLDB queries
4. Agent states: "I INSPECTED runtime state and found the bug"

---

## What You've Achieved

You've just run the first AI iOS debugging session:

- AI built iOS code
- AI attached to live app via LLDB
- AI queried actual Swift runtime state
- AI found bug through inspection
- AI verified fix with proof

**The feedback loop is closed for iOS. The endoskeleton reaches Swift.**

---

## Next Steps

- Document your debugging session
- Share the LLDB output
- Try more complex iOS apps
- Combine with CDP for full-stack debugging
- Extend to macOS apps

---

*Welcome to LLDB-assisted AI development.*
