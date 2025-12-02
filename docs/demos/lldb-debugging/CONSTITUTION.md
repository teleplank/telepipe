# LLDB Debugging Demo: Technical Constitution
## Specification for TodoApp with Intentional Bug

---

## Overview

A simple iOS TodoApp built in SwiftUI with one critical addition: **an intentional persistence bug for the AI to discover via LLDB**.

---

## Application Specification

**Name:** TodoApp
**Platform:** iOS 17+ (Swift/SwiftUI)
**Purpose:** Demonstrate LLDB debugging via Telepipe
**Bug:** Items don't persist between app launches

---

## Functional Requirements

### Core Features
1. Add todo items with text input
2. Display list of todos
3. Toggle todo completion status
4. Visual indication of completed items (strikethrough)

### The Intentional Bug
Items don't persist between app launches because:
- `saveItems()` method exists but is never called
- `loadItems()` doesn't actually load from UserDefaults

**This bug is discoverable ONLY via runtime inspection - the code compiles and runs fine.**

---

## Technical Architecture

### File Structure
```
TodoApp/
├── TodoApp.swift           # @main App entry
├── ContentView.swift       # Main UI
├── TodoViewModel.swift     # State management (BUGGY)
└── TodoItem.swift          # Data model
```

### Component Specifications

#### TodoItem.swift
```swift
import Foundation

struct TodoItem: Identifiable, Codable, Equatable {
    let id: UUID
    var text: String
    var isCompleted: Bool

    init(id: UUID = UUID(), text: String, isCompleted: Bool = false) {
        self.id = id
        self.text = text
        self.isCompleted = isCompleted
    }
}
```

#### TodoViewModel.swift (BUGGY VERSION)
```swift
import Foundation
import SwiftUI

class TodoViewModel: ObservableObject {
    @Published var items: [TodoItem] = []

    private let storageKey = "todos"

    func addItem(_ text: String) {
        let item = TodoItem(text: text)
        items.append(item)
        // BUG: saveItems() should be called here but isn't!
    }

    func toggleItem(_ item: TodoItem) {
        if let index = items.firstIndex(where: { $0.id == item.id }) {
            items[index].isCompleted.toggle()
            // BUG: saveItems() should be called here but isn't!
        }
    }

    func deleteItem(_ item: TodoItem) {
        items.removeAll { $0.id == item.id }
        // BUG: saveItems() should be called here but isn't!
    }

    func loadItems() {
        // BUG: This should load from UserDefaults but doesn't!
        // The method exists but does nothing useful
        items = []
    }

    // This method exists but is NEVER CALLED
    private func saveItems() {
        if let encoded = try? JSONEncoder().encode(items) {
            UserDefaults.standard.set(encoded, forKey: storageKey)
        }
    }

    // This method exists but loadItems() doesn't use it
    private func loadFromStorage() -> [TodoItem] {
        guard let data = UserDefaults.standard.data(forKey: storageKey),
              let decoded = try? JSONDecoder().decode([TodoItem].self, from: data) else {
            return []
        }
        return decoded
    }
}
```

#### TodoViewModel.swift (FIXED VERSION)
```swift
import Foundation
import SwiftUI

class TodoViewModel: ObservableObject {
    @Published var items: [TodoItem] = []

    private let storageKey = "todos"

    func addItem(_ text: String) {
        let item = TodoItem(text: text)
        items.append(item)
        saveItems()  // FIX: Now saves!
    }

    func toggleItem(_ item: TodoItem) {
        if let index = items.firstIndex(where: { $0.id == item.id }) {
            items[index].isCompleted.toggle()
            saveItems()  // FIX: Now saves!
        }
    }

    func deleteItem(_ item: TodoItem) {
        items.removeAll { $0.id == item.id }
        saveItems()  // FIX: Now saves!
    }

    func loadItems() {
        items = loadFromStorage()  // FIX: Now actually loads!
    }

    private func saveItems() {
        if let encoded = try? JSONEncoder().encode(items) {
            UserDefaults.standard.set(encoded, forKey: storageKey)
        }
    }

    private func loadFromStorage() -> [TodoItem] {
        guard let data = UserDefaults.standard.data(forKey: storageKey),
              let decoded = try? JSONDecoder().decode([TodoItem].self, from: data) else {
            return []
        }
        return decoded
    }
}
```

#### ContentView.swift
```swift
import SwiftUI

struct ContentView: View {
    @StateObject private var viewModel = TodoViewModel()
    @State private var newTodoText = ""

    var body: some View {
        NavigationView {
            VStack {
                // Input area
                HStack {
                    TextField("New todo", text: $newTodoText)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .padding(.leading)

                    Button(action: {
                        if !newTodoText.isEmpty {
                            viewModel.addItem(newTodoText)
                            newTodoText = ""
                        }
                    }) {
                        Image(systemName: "plus.circle.fill")
                            .font(.title)
                    }
                    .padding(.trailing)
                }
                .padding(.vertical)

                // Todo list
                List {
                    ForEach(viewModel.items) { item in
                        HStack {
                            Image(systemName: item.isCompleted ? "checkmark.circle.fill" : "circle")
                                .foregroundColor(item.isCompleted ? .green : .gray)

                            Text(item.text)
                                .strikethrough(item.isCompleted)
                                .foregroundColor(item.isCompleted ? .gray : .primary)

                            Spacer()
                        }
                        .contentShape(Rectangle())
                        .onTapGesture {
                            viewModel.toggleItem(item)
                        }
                    }
                    .onDelete { indexSet in
                        indexSet.forEach { index in
                            viewModel.deleteItem(viewModel.items[index])
                        }
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

---

## LLDB Verification Points

These are the specific queries the agent MUST execute to find the bug:

### Verification 1: Initial State
```
(lldb) po viewModel.items.count
0
```
Expected: 0 items initially

### Verification 2: After Adding Item
```
(lldb) po viewModel.items.count
1

(lldb) po viewModel.items[0].text
"Buy milk"
```
Expected: 1 item with correct text

### Verification 3: After Adding Second Item
```
(lldb) po viewModel.items.count
2
```
Expected: 2 items

### Verification 4: Check UserDefaults (Bug Indicator)
```
(lldb) po UserDefaults.standard.object(forKey: "todos")
nil
```
Expected (bug present): `nil` - items weren't saved!

### Verification 5: After App Restart
```
(lldb) po viewModel.items.count
0
```
**BUG CONFIRMED:** Items are gone after restart!

### Verification 6: After Fix - Items Persist
```
(lldb) po viewModel.items.count
2

(lldb) po UserDefaults.standard.object(forKey: "todos")
<NSData: 0x...> {length = 123}
```
Expected: Items persist, UserDefaults contains data

---

## Debugging Commands Reference

| Command | Purpose | Expected Use |
|---------|---------|--------------|
| `po viewModel` | Print view model | See full object state |
| `po viewModel.items` | Print items array | See all todos |
| `po viewModel.items.count` | Count items | Verify add/delete |
| `po viewModel.items[0]` | Print first item | Inspect specific todo |
| `po viewModel.items[0].text` | Get item text | Verify content |
| `po viewModel.items[0].isCompleted` | Get completion | Verify toggle |
| `po UserDefaults.standard.object(forKey: "todos")` | Check storage | Verify persistence |
| `expr viewModel.items.count` | Evaluate count | Same as po but returns value |
| `bt` | Backtrace | See call stack |
| `frame variable` | Local variables | See current scope |

---

## Exit Criteria

The debugging session is complete when:

1. [ ] App builds and runs without errors
2. [ ] Agent verified items.count increases after add
3. [ ] Agent discovered items.count === 0 after restart
4. [ ] Agent identified missing saveItems() calls
5. [ ] Agent fixed the bug
6. [ ] Agent verified fix via LLDB (items persist after restart)
7. [ ] Debugging report produced with actual LLDB output

---

## Invariants

These must ALWAYS be true:

1. `items` is never nil (initialized as empty array)
2. Each `TodoItem` has unique `id`
3. `items.count` reflects actual array length
4. After fix: `UserDefaults.standard.object(forKey: "todos")` !== nil when items exist
5. After fix: `items.count` is same before and after restart

---

## Bug Discovery Path

The agent should follow this debugging path:

```
1. Launch app (empty state)
   → po viewModel.items.count → 0 ✅

2. Add "Buy milk"
   → po viewModel.items.count → 1 ✅
   → po viewModel.items[0].text → "Buy milk" ✅

3. Add "Walk dog"
   → po viewModel.items.count → 2 ✅

4. Check persistence
   → po UserDefaults.standard.object(forKey: "todos") → nil ❌
   → "Hmm, nothing saved to UserDefaults..."

5. Restart app
   → po viewModel.items.count → 0 ❌
   → "BUG FOUND! Items disappeared!"

6. Inspect code
   → Find saveItems() exists but never called
   → Find loadItems() doesn't actually load

7. Fix code
   → Add saveItems() calls
   → Fix loadItems() to use loadFromStorage()

8. Verify fix
   → Add items → Restart → po viewModel.items.count → 2 ✅
   → "BUG FIXED! Items now persist!"
```

---

*This specification defines a debuggable app. The agent discovers the bug via LLDB inspection, not guessing.*
