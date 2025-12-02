# LLDB Debugging Agent Prompt
## Debug iOS Apps Like a Sighted Developer

---

## Preamble: What Makes This Different

You are about to debug an iOS application in a way no AI has done before.

Traditionally, AI writes iOS code and hopes it works. The feedback loop is open:
- Write Swift → Compile → Hope → Ship

Today, you close the loop. You will:
- Write Swift → Attach LLDB → Query actual runtime state → VERIFY → Ship with proof

This is possible because you have access to **LLDB via Telepipe**. You can reach INTO the Swift runtime and query actual object values - view model state, array contents, UserDefaults data.

**This debugging is impossible without LLDB.**

---

## Step 1: Understand Your Capabilities

You have access to LLDB (LLVM Debugger) via Telepipe:

```bash
# Attach to running iOS app
telepipe redirect --id lldb -- lldb -n TodoApp

# Send LLDB commands
echo "po viewModel.items" | telepipe exec --id lldb
echo "bt" | telepipe exec --id lldb
echo "frame variable" | telepipe exec --id lldb
```

**Key LLDB commands you can execute:**

| Command | What It Does |
|---------|--------------|
| `po <expr>` | Print object (shows description) |
| `p <expr>` | Print value (shows raw value) |
| `po viewModel.items` | Print the items array |
| `po viewModel.items.count` | Get array count |
| `po viewModel.items[0].text` | Get first item's text |
| `bt` | Backtrace (call stack) |
| `frame variable` | All local variables |
| `expr <code>` | Evaluate Swift expression |

**You can see inside the Swift runtime. Use this power.**

---

## Step 2: The Application - TodoApp

You will build (or be given) a TodoApp with these features:

1. **Add todo items** - Text input + Add button
2. **Display todos** - List view showing all items
3. **Toggle completion** - Tap to mark complete/incomplete
4. **Persistence** - Items should survive app restart

### The Catch

**The app has an intentional bug: items don't persist between launches.**

The `saveItems()` method exists but is never called. The `loadItems()` method exists but doesn't actually load.

**Your mission: Find this bug using LLDB inspection.**

---

## Step 3: The Debugging Session

### Phase 1: Verify Initial State

After app launches, query the view model:

```
(lldb) po viewModel.items.count
```

Expected: `0` (no items yet)

### Phase 2: Add Items and Verify

Add "Buy milk" via the UI, then query:

```
(lldb) po viewModel.items.count
```

Expected: `1`

```
(lldb) po viewModel.items[0].text
```

Expected: `"Buy milk"`

Add another item "Walk dog":

```
(lldb) po viewModel.items.count
```

Expected: `2`

### Phase 3: Check Persistence (Find the Bug)

Query UserDefaults to see if items were saved:

```
(lldb) po UserDefaults.standard.object(forKey: "todos")
```

**If you see `nil` - that's the bug! Items aren't being saved.**

### Phase 4: Restart and Confirm Bug

1. Stop the app (Cmd+. in Xcode or kill process)
2. Relaunch the app
3. Re-attach LLDB via Telepipe
4. Query items:

```
(lldb) po viewModel.items.count
```

**If you see `0` when it should be `2` - BUG CONFIRMED!**

### Phase 5: Diagnose the Bug

Inspect the code. You'll find:
- `saveItems()` method exists but is never called
- `loadItems()` sets `items = []` instead of loading from storage

### Phase 6: Fix the Bug

Modify `TodoViewModel.swift`:

```swift
func addItem(_ text: String) {
    let item = TodoItem(text: text)
    items.append(item)
    saveItems()  // ADD THIS LINE
}

func toggleItem(_ item: TodoItem) {
    if let index = items.firstIndex(where: { $0.id == item.id }) {
        items[index].isCompleted.toggle()
        saveItems()  // ADD THIS LINE
    }
}

func loadItems() {
    items = loadFromStorage()  // CHANGE THIS LINE
}
```

### Phase 7: Verify the Fix

1. Rebuild and run
2. Add items
3. Query via LLDB: items present
4. Restart app
5. Query via LLDB: **items still present!**

```
(lldb) po viewModel.items.count
2

(lldb) po UserDefaults.standard.object(forKey: "todos")
<NSData: 0x...>
```

**BUG FIXED AND VERIFIED VIA LLDB!**

---

## Critical Requirements

### You MUST Use LLDB

Don't assume behavior. Query actual runtime state.

**WRONG:**
```
"The items array should have 2 elements after adding two todos."
```

**CORRECT:**
```
"I queried via LLDB:
(lldb) po viewModel.items.count
2

Verified: items array has exactly 2 elements."
```

### You MUST Find the Bug Via Inspection

Don't guess the bug from reading code. Discover it through LLDB:

1. Add items → verify they exist (LLDB shows count > 0)
2. Restart app → verify they're gone (LLDB shows count = 0)
3. Check UserDefaults → verify nothing saved (LLDB shows nil)
4. Conclude: "Persistence bug found via runtime inspection"

### You MUST Verify the Fix

After fixing:

1. Add items
2. LLDB query: `po viewModel.items.count` → 2
3. Restart app
4. LLDB query: `po viewModel.items.count` → 2 (still there!)
5. Conclude: "Fix verified via LLDB - items persist"

---

## Debugging Report Template

Produce this report after your session:

```markdown
# TodoApp Debugging Report

## Build Info
- Date: [DATE]
- Agent: Claude Opus
- Method: LLDB via Telepipe

## Debugging Session

### 1. Initial State ✅
**LLDB:** `po viewModel.items.count`
**Result:** `0`
**Status:** App starts with empty state

### 2. After Adding "Buy milk" ✅
**LLDB:** `po viewModel.items.count`
**Result:** `1`
**LLDB:** `po viewModel.items[0].text`
**Result:** `"Buy milk"`
**Status:** Item added correctly

### 3. After Adding "Walk dog" ✅
**LLDB:** `po viewModel.items.count`
**Result:** `2`
**Status:** Second item added

### 4. Persistence Check ❌
**LLDB:** `po UserDefaults.standard.object(forKey: "todos")`
**Result:** `nil`
**Status:** BUG - Items not saved to UserDefaults!

### 5. After Restart ❌
**LLDB:** `po viewModel.items.count`
**Result:** `0`
**Status:** BUG CONFIRMED - Items don't persist!

## Bug Analysis

**Symptom:** Items disappear after app restart
**Root Cause:** `saveItems()` never called; `loadItems()` doesn't load
**Evidence:** UserDefaults.object() returns nil; items.count = 0 after restart

## Fix Applied

```swift
// Added saveItems() calls to addItem() and toggleItem()
// Changed loadItems() to call loadFromStorage()
```

## Fix Verification ✅

### After Fix - Add Items
**LLDB:** `po viewModel.items.count`
**Result:** `2`

### After Fix - Restart
**LLDB:** `po viewModel.items.count`
**Result:** `2` (items persist!)

**LLDB:** `po UserDefaults.standard.object(forKey: "todos")`
**Result:** `<NSData: 0x...>` (data saved!)

## Conclusion

Bug found and fixed via LLDB runtime inspection:
- Discovered missing persistence through actual state queries
- Verified fix through actual state queries
- **This debugging was impossible without LLDB access**
```

---

## Remember

You're not hoping it works. You're PROVING it works via LLDB.

**The endoskeleton is real. The feedback loop closes. Use your sight.**

---

## Quick Reference: LLDB Commands

```bash
# Inspect objects
po viewModel                    # Full view model
po viewModel.items              # Items array
po viewModel.items.count        # Count
po viewModel.items[0]           # First item
po viewModel.items[0].text      # First item's text

# Check persistence
po UserDefaults.standard.object(forKey: "todos")

# Debugging
bt                              # Backtrace
frame variable                  # Local variables
expr viewModel.items.append(TodoItem(text: "test"))  # Execute code

# Navigation
thread list                     # All threads
frame select 2                  # Jump to frame
```

---

*Read this prompt. Internalize your LLDB capabilities. Debug. Verify. Prove.*
