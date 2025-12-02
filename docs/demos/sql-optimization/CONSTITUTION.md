# Constitution: SQL Query Optimization Demo
## Technical Specification for the Demo Database

---

## Purpose

This document defines the exact database schema, data volumes, and expected optimization targets for the SQL Query Optimization demo. It serves as the source of truth for both setup and verification.

---

## Database Specification

### Database Name
```
ecommerce_demo
```

### PostgreSQL Version
```
PostgreSQL 14+ (required for optimal EXPLAIN ANALYZE output)
```

---

## Schema Definition

### Table: users

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255),
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW()
);

-- INTENTIONALLY NO INDEXES (except primary key)
-- This creates the optimization opportunity
```

**Row count:** 100,000 rows

**Data distribution:**
- ~33% @gmail.com emails
- ~33% @yahoo.com emails
- ~33% @company.com emails

**Expected slow query:**
```sql
SELECT * FROM users WHERE email LIKE '%@gmail.com';
-- Expected: Seq Scan, 5000+ ms
```

**Expected optimization:**
```sql
CREATE INDEX idx_users_email ON users USING btree(email text_pattern_ops);
-- Result: Bitmap Index Scan, ~50 ms
-- Improvement: 100x
```

---

### Table: orders

```sql
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    total DECIMAL(10,2),
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);

-- INTENTIONALLY NO INDEXES (except primary key)
-- No foreign key constraint - allows faster bulk insert
```

**Row count:** 500,000 rows

**Data distribution:**
- user_id: Random 1-100000
- total: Random $10-$510
- status: 25% each of pending/processing/shipped/delivered
- created_at: Random within last year

**Expected slow query:**
```sql
SELECT * FROM orders
WHERE user_id = 12345
ORDER BY created_at DESC
LIMIT 10;
-- Expected: Seq Scan, 8000+ ms
```

**Expected optimization:**
```sql
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);
-- Result: Index Scan, ~0.1 ms
-- Improvement: 67,000x
```

---

### Table: products

```sql
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255),
    price DECIMAL(10,2),
    category VARCHAR(100)
);

-- INTENTIONALLY NO FULL-TEXT INDEX
```

**Row count:** 10,000 rows

**Data distribution:**
- name: "Laptop N", "Phone N", "Tablet N", "Monitor N", "Keyboard N"
- price: Random $50-$2050
- category: Electronics/Computers/Accessories/Peripherals

**Expected slow query:**
```sql
SELECT * FROM products WHERE name ILIKE '%laptop%';
-- Expected: Seq Scan, 2000+ ms
```

**Expected optimization:**
```sql
CREATE INDEX idx_products_name_gin ON products
USING gin(to_tsvector('english', name));

-- Query changes to:
SELECT * FROM products
WHERE to_tsvector('english', name) @@ to_tsquery('laptop');
-- Result: Bitmap Index Scan, ~45 ms
-- Improvement: 54x
```

---

## Data Generation Scripts

### Full Setup Script

```sql
-- Run this in psql after: createdb ecommerce_demo

-- Create tables
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255),
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    total DECIMAL(10,2),
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255),
    price DECIMAL(10,2),
    category VARCHAR(100)
);

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

-- Verify counts
SELECT 'users' as table_name, COUNT(*) FROM users
UNION ALL
SELECT 'orders', COUNT(*) FROM orders
UNION ALL
SELECT 'products', COUNT(*) FROM products;
```

---

## Reset Script

To reset the database for a fresh demo run:

```sql
-- Drop all non-primary-key indexes
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_orders_user_id;
DROP INDEX IF EXISTS idx_orders_created_at;
DROP INDEX IF EXISTS idx_products_name_gin;

-- Verify indexes are gone
SELECT indexname FROM pg_indexes
WHERE tablename IN ('users', 'orders', 'products')
AND indexname NOT LIKE '%pkey';
-- Should return 0 rows
```

---

## Verification Queries

### Pre-Optimization Verification

All of these should show "Seq Scan":

```sql
EXPLAIN SELECT * FROM users WHERE email LIKE '%@gmail.com';
-- Should show: Seq Scan on users

EXPLAIN SELECT * FROM orders WHERE user_id = 12345;
-- Should show: Seq Scan on orders

EXPLAIN SELECT * FROM products WHERE name ILIKE '%laptop%';
-- Should show: Seq Scan on products
```

### Post-Optimization Verification

All of these should show "Index Scan" or "Bitmap Index Scan":

```sql
EXPLAIN SELECT * FROM users WHERE email LIKE '%@gmail.com';
-- Should show: Bitmap Heap Scan + Bitmap Index Scan

EXPLAIN SELECT * FROM orders WHERE user_id = 12345;
-- Should show: Index Scan using idx_orders_user_id

EXPLAIN SELECT * FROM products
WHERE to_tsvector('english', name) @@ to_tsquery('laptop');
-- Should show: Bitmap Heap Scan + Bitmap Index Scan
```

---

## Expected Optimization Results

| Query | Pre-Optimization | Post-Optimization | Improvement |
|-------|------------------|-------------------|-------------|
| Email search | Seq Scan, ~5000ms | Bitmap Index Scan, ~50ms | **100x** |
| Order history | Seq Scan, ~8000ms | Index Scan, ~0.1ms | **67,000x** |
| Product search | Seq Scan, ~2000ms | Bitmap Index Scan, ~45ms | **54x** |

---

## Index Types Reference

The agent should know when to use each index type:

| Index Type | Use Case | Example |
|------------|----------|---------|
| **B-tree** | Equality, range, ORDER BY | `user_id = 12345` |
| **B-tree + text_pattern_ops** | LIKE patterns (prefix) | `email LIKE 'user%'` |
| **GIN** | Full-text search, arrays | `to_tsvector @@ to_tsquery` |
| **GIST** | Geometric, range types | PostGIS, tsrange |

---

## Telepipe Connection

```bash
# Connect Telepipe to PostgreSQL
telepipe connect --id db --port 5432

# Verify connection
telepipe info --id db

# Execute SQL via Telepipe
telepipe exec --id db
# Then type SQL commands
```

---

## Success Criteria

The demo is successful when the agent:

1. **Runs EXPLAIN ANALYZE** before optimization
2. **Identifies "Seq Scan"** as the bottleneck
3. **Creates appropriate indexes** (correct types)
4. **Runs EXPLAIN ANALYZE** after optimization
5. **Confirms "Index Scan"** in output
6. **Reports actual time improvement** with metrics

---

## Appendix: Why These Numbers?

### 100,000 users
- Large enough for noticeable Seq Scan times
- Small enough for fast demo setup
- Realistic for medium-sized application

### 500,000 orders
- 5x users = realistic order volume
- Makes user_id index impact dramatic
- Demonstrates ORDER BY optimization need

### 10,000 products
- Enough for full-text search demonstration
- GIN index benefit clearly visible
- Fast to generate and query

---

*This constitution defines the demo. All behavior derives from this specification.*
