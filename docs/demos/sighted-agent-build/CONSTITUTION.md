# Tic-Tac-Toe: Constitution
## Technical Specification for Sighted Agent Build

---

## Overview

A simple Tic-Tac-Toe game built in React with one critical addition: **state exposure for CDP verification**.

---

## Game Rules

1. Two players: X and O
2. X always goes first
3. Players alternate turns
4. First player to get 3 in a row (horizontal, vertical, diagonal) wins
5. If all 9 cells are filled with no winner, it's a draw

---

## Technical Requirements

### Framework
- React 18+ with hooks
- Create React App or Vite

### File Structure
```
tic-tac-toe/
├── src/
│   ├── App.js          # Main game logic
│   ├── App.css         # Styles
│   ├── Board.js        # 3x3 grid component
│   ├── Cell.js         # Individual cell component
│   └── index.js        # Entry point
├── package.json
└── README.md
```

---

## State Specification

### Core State Shape

```javascript
{
  board: Array(9),      // [null|'X'|'O', ...]
  xIsNext: boolean,     // true = X's turn, false = O's turn
  winner: null|'X'|'O'  // null = no winner yet
}
```

### Cell Indices
```
 0 | 1 | 2
-----------
 3 | 4 | 5
-----------
 6 | 7 | 8
```

### Winning Combinations
```javascript
const WINNING_LINES = [
  [0, 1, 2],  // top row
  [3, 4, 5],  // middle row
  [6, 7, 8],  // bottom row
  [0, 3, 6],  // left column
  [1, 4, 7],  // middle column
  [2, 5, 8],  // right column
  [0, 4, 8],  // diagonal top-left to bottom-right
  [2, 4, 6]   // diagonal top-right to bottom-left
];
```

---

## Component Specifications

### App.js

**Responsibilities:**
- Manage game state (board, xIsNext, winner)
- Implement move logic
- Implement win detection
- Implement reset functionality
- **CRITICAL: Expose state to window for CDP access**

**State Exposure (Required):**
```javascript
useEffect(() => {
  window.__GAME_STATE__ = { board, xIsNext, winner };
}, [board, xIsNext, winner]);
```

**Functions:**
- `handleClick(index)` - Process a move
- `calculateWinner(board)` - Check for winner
- `resetGame()` - Reset to initial state

### Board.js

**Props:**
- `board: Array(9)` - Current board state
- `onClick: (index) => void` - Click handler
- `disabled: boolean` - Disable clicks when game over

**Renders:**
- 3x3 grid of Cell components

### Cell.js

**Props:**
- `value: null|'X'|'O'` - Cell content
- `onClick: () => void` - Click handler
- `disabled: boolean` - Disable if occupied or game over

**Renders:**
- Button showing X, O, or empty
- CSS class: `.cell`

---

## Styling Requirements

### Board
```css
.board {
  display: grid;
  grid-template-columns: repeat(3, 100px);
  grid-template-rows: repeat(3, 100px);
  gap: 4px;
}
```

### Cell
```css
.cell {
  width: 100px;
  height: 100px;
  font-size: 48px;
  font-weight: bold;
  cursor: pointer;
  background: #f0f0f0;
  border: 2px solid #333;
}

.cell:hover:not(:disabled) {
  background: #e0e0e0;
}

.cell:disabled {
  cursor: not-allowed;
}

.cell.x {
  color: #3B82F6;  /* blue */
}

.cell.o {
  color: #EF4444;  /* red */
}
```

### Status Display
```css
.status {
  margin: 20px 0;
  font-size: 24px;
  font-weight: bold;
}
```

### Reset Button
```css
.reset-button {
  padding: 10px 20px;
  font-size: 18px;
  cursor: pointer;
}
```

---

## Behavioral Specifications

### Initial State
- board: `[null, null, null, null, null, null, null, null, null]`
- xIsNext: `true`
- winner: `null`
- Status displays: "Next player: X"

### On Cell Click
1. If cell occupied → ignore
2. If winner exists → ignore
3. Place current player's mark in cell
4. Check for winner
5. Toggle xIsNext
6. Update status display

### On Win
- Set winner state
- Status displays: "Winner: X" or "Winner: O"
- Disable further moves

### On Draw
- All cells filled, no winner
- Status displays: "Draw!"
- Disable further moves

### On Reset
- Restore initial state
- Enable all cells

---

## CDP Verification Points

These are the specific queries the agent MUST execute:

### Verification 1: Initial Load
```javascript
// Query
Runtime.evaluate({
  expression: "window.__GAME_STATE__",
  returnByValue: true
})

// Expected
{
  board: [null, null, null, null, null, null, null, null, null],
  xIsNext: true,
  winner: null
}
```

### Verification 2: After X's First Move (cell 0)
```javascript
// Action
Runtime.evaluate({
  expression: "document.querySelectorAll('.cell')[0].click()"
})

// Query
Runtime.evaluate({
  expression: "window.__GAME_STATE__",
  returnByValue: true
})

// Expected
{
  board: ['X', null, null, null, null, null, null, null, null],
  xIsNext: false,
  winner: null
}
```

### Verification 3: After O's First Move (cell 4)
```javascript
// Action
Runtime.evaluate({
  expression: "document.querySelectorAll('.cell')[4].click()"
})

// Query
Runtime.evaluate({
  expression: "window.__GAME_STATE__",
  returnByValue: true
})

// Expected
{
  board: ['X', null, null, null, 'O', null, null, null, null],
  xIsNext: true,
  winner: null
}
```

### Verification 4: Win Condition (X wins top row)
```javascript
// Play sequence: X(1), O(5), X(2) - X wins with [0,1,2]
Runtime.evaluate({
  expression: `
    const cells = document.querySelectorAll('.cell');
    cells[1].click(); // X
    cells[5].click(); // O
    cells[2].click(); // X wins
  `
})

// Query
Runtime.evaluate({
  expression: "window.__GAME_STATE__",
  returnByValue: true
})

// Expected
{
  board: ['X', 'X', 'X', null, 'O', 'O', null, null, null],
  xIsNext: false,
  winner: 'X'
}
```

### Verification 5: Console Clean
```javascript
// Enable console monitoring
Console.enable()

// Any Console.messageAdded event with level "error" = FAIL
// Expected: 0 error events
```

---

## Exit Criteria

The build is complete when:

1. ✅ App renders without errors
2. ✅ Initial state verified via CDP
3. ✅ Move mechanics verified via CDP
4. ✅ Turn alternation verified via CDP
5. ✅ Win detection verified via CDP
6. ✅ Zero console errors
7. ✅ Verification report produced

---

## Invariants

These must ALWAYS be true:

1. `board.length === 9`
2. `board[i]` is one of: `null`, `'X'`, `'O'`
3. After X moves, `xIsNext === false`
4. After O moves, `xIsNext === true`
5. `winner` only set when winning condition met
6. Once winner set, no more moves accepted
7. `window.__GAME_STATE__` always reflects current state

---

*This specification defines what to build. The agent verifies correctness via CDP queries against actual runtime state.*
