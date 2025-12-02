# Quick Start: SQL Query Optimization Demo
## Get Optimizing in 10 Minutes

---

## Prerequisites

- **PostgreSQL 14+** installed and running
- **Telepipe** installed (`npm install -g telepipe`)
- **AI Agent** (Claude Opus, Claude Sonnet, or similar)

---

## Step 1: Create the Demo Database (3 minutes)

```bash
# Create the database
createdb ecommerce_demo

# Connect and create tables
psql ecommerce_demo << 'EOF'

-- Users table (will have 100k rows)
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255),
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Orders table (will have 500k rows)
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    total DECIMAL(10,2),
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Products table (will have 10k rows)
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255),
    price DECIMAL(10,2),
    category VARCHAR(100)
);

EOF
```

---

## Step 2: Load Sample Data (2 minutes)

```bash
psql ecommerce_demo << 'EOF'

-- Generate 100k users
INSERT INTO users (email, name, created_at)
SELECT
    'user' || n || CASE (random() * 3)::int
        WHEN 0 THEN '@gmail.com'
        WHEN 1 THEN '@yahoo.com'
        ELSE '@company.com'
    END,
    'User ' || n,
    NOW() - (random() * 365 * interval '1 day')
FROM generate_series(1, 100000) n;

-- Generate 500k orders
INSERT INTO orders (user_id, total, status, created_at)
SELECT
    (random() * 99999 + 1)::int,
    (random() * 500 + 10)::decimal(10,2),
    CASE (random() * 4)::int
        WHEN 0 THEN 'pending'
        WHEN 1 THEN 'processing'
        WHEN 2 THEN 'shipped'
        ELSE 'delivered'
    END,
    NOW() - (random() * 365 * interval '1 day')
FROM generate_series(1, 500000) n;

-- Generate 10k products
INSERT INTO products (name, price, category)
SELECT
    CASE (random() * 5)::int
        WHEN 0 THEN 'Laptop '
        WHEN 1 THEN 'Phone '
        WHEN 2 THEN 'Tablet '
        WHEN 3 THEN 'Monitor '
        ELSE 'Keyboard '
    END || n,
    (random() * 2000 + 50)::decimal(10,2),
    CASE (random() * 4)::int
        WHEN 0 THEN 'Electronics'
        WHEN 1 THEN 'Computers'
        WHEN 2 THEN 'Accessories'
        ELSE 'Peripherals'
    END
FROM generate_series(1, 10000) n;

-- Verify data loaded
SELECT 'users' as table_name, COUNT(*) FROM users
UNION ALL
SELECT 'orders', COUNT(*) FROM orders
UNION ALL
SELECT 'products', COUNT(*) FROM products;

EOF
```

**Expected output:**
```
 table_name | count
------------+--------
 users      | 100000
 orders     | 500000
 products   |  10000
```

---

## Step 3: Connect Telepipe (1 minute)

```bash
# Connect Telepipe to PostgreSQL (direct TCP - no websocat needed!)
telepipe connect --id db --port 5432

# Verify connection
telepipe info --id db
```

**Expected output:**
```
Session: db
Type: connect
Target: localhost:5432
Status: connected
```

---

## Step 4: Test a Slow Query (1 minute)

Before your agent starts, verify the setup is working:

```bash
# This should be SLOW (no indexes!)
psql ecommerce_demo -c "EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%@gmail.com';"
```

**Expected output (slow):**
```
                                                    QUERY PLAN
------------------------------------------------------------------------------------------------------------------
 Seq Scan on users  (cost=0.00..2931.00 rows=... width=...) (actual time=...rows=... loops=1)
   Filter: ((email)::text ~~ '%@gmail.com'::text)
 Execution Time: XXXX.XXX ms  <-- Should be 1000+ ms
```

**If you see "Seq Scan" and high execution time, you're ready!**

---

## Step 5: Give Your Agent the Prompts (2 minutes)

Your agent needs these files:

1. **[docs/for-agents/README.md](../../../docs/for-agents/README.md)** - Telepipe capabilities
2. **[SQL_AGENT_PROMPT.md](./SQL_AGENT_PROMPT.md)** - SQL optimization instructions
3. **[CONSTITUTION.md](./CONSTITUTION.md)** - Demo database specification

**Tell your agent:**

```
You now have access to a PostgreSQL database via Telepipe.

Read these files to understand your capabilities:
- docs/for-agents/README.md (Telepipe basics)
- docs/demos/sql-optimization/SQL_AGENT_PROMPT.md (your mission)
- docs/demos/sql-optimization/CONSTITUTION.md (database spec)

Your mission: Optimize the slow queries in ecommerce_demo.
Use EXPLAIN ANALYZE before AND after each optimization.
PROVE your improvements with actual execution times.
```

---

## Step 6: Watch the Magic (5+ minutes)

Your agent will:

1. **Connect** to PostgreSQL via Telepipe
2. **Identify** slow queries using EXPLAIN ANALYZE
3. **Create** appropriate indexes
4. **Verify** improvements with before/after metrics
5. **Report** proven speedups (100x+)

---

## Expected Agent Behavior

### Phase 1: Discovery
```sql
-- Agent runs EXPLAIN ANALYZE on common queries
EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%@gmail.com';
-- Agent sees: Seq Scan, 5000+ ms

EXPLAIN ANALYZE SELECT * FROM orders WHERE user_id = 12345;
-- Agent sees: Seq Scan, 8000+ ms

EXPLAIN ANALYZE SELECT * FROM products WHERE name ILIKE '%laptop%';
-- Agent sees: Seq Scan, 2000+ ms
```

### Phase 2: Optimization
```sql
-- Agent creates appropriate indexes
CREATE INDEX idx_users_email ON users USING btree(email text_pattern_ops);
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_products_name_gin ON products USING gin(to_tsvector('english', name));
```

### Phase 3: Verification
```sql
-- Agent re-runs EXPLAIN ANALYZE
EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%@gmail.com';
-- Agent sees: Bitmap Index Scan, 50 ms
-- Agent reports: "100x faster! Verified."
```

---

## Troubleshooting

### PostgreSQL not running
```bash
# macOS
brew services start postgresql

# Linux
sudo systemctl start postgresql
```

### Database doesn't exist
```bash
createdb ecommerce_demo
```

### Telepipe can't connect
```bash
# Check PostgreSQL is listening on 5432
lsof -i :5432

# Reconnect
telepipe disconnect --id db
telepipe connect --id db --port 5432
```

### Queries already fast
```bash
# Drop all indexes and try again
psql ecommerce_demo -c "DROP INDEX IF EXISTS idx_users_email, idx_orders_user_id, idx_orders_created_at, idx_products_name_gin;"
```

---

## Success Criteria

The demo is successful when your agent:

1. **Measures** query performance with EXPLAIN ANALYZE
2. **Identifies** Seq Scan bottlenecks
3. **Creates** appropriate indexes
4. **Verifies** Index Scan usage
5. **Reports** proven speedups with metrics

**Example successful output:**
```
Optimization Complete!

| Query | Before | After | Improvement |
|-------|--------|-------|-------------|
| Email search | 5234ms | 52ms | 100x |
| Order history | 8123ms | 0.12ms | 67,000x |
| Product search | 2456ms | 45ms | 54x |

All improvements verified with EXPLAIN ANALYZE.
```

---

## Cleanup

```bash
# Stop Telepipe session
telepipe disconnect --id db

# Drop demo database (optional)
dropdb ecommerce_demo
```

---

## Next Steps

- Read [CASE_STUDY.md](./CASE_STUDY.md) for the full experiment documentation
- Try the [Sighted Agent Build](../sighted-agent-build/) (CDP demo)
- Try the [LLDB Debugging](../lldb-debugging/) (iOS demo)

---

*You're ready to watch AI optimize SQL with proof. Have fun!*
