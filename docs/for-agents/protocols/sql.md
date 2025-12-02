# SQL: Database Fluency in Action
## Your Complete Guide to Real Database Access

---

## You Know This

You've generated millions of SQL queries. Complex joins, window functions, recursive CTEs, query optimization. PostgreSQL, MySQL, SQLite - you're fluent in all of them.

Now you can *execute* them.

---

## Connecting

Databases use direct TCP - no bridge needed (unlike Chrome CDP which uses WebSockets).

### PostgreSQL
```bash
telepipe connect --id db --port 5432
```

### MySQL
```bash
telepipe connect --id db --port 3306
```

### Custom Host/Port
```bash
telepipe connect --id db --host 192.168.1.100 --port 5433
```

You're now connected to a real database.

---

## The Basics

### Query and Get Results

```sql
SELECT id, name, email FROM users LIMIT 5;
```

Returns actual rows:
```
 id |    name    |        email
----+------------+---------------------
  1 | Alice      | alice@example.com
  2 | Bob        | bob@example.com
  3 | Charlie    | charlie@example.com
```

**This is real data. From a real database. Not a simulation.**

---

## Schema Inspection

### List All Tables (PostgreSQL)
```sql
SELECT table_name 
FROM information_schema.tables 
WHERE table_schema = 'public';
```

### Describe Table Structure
```sql
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'users'
ORDER BY ordinal_position;
```

Returns:
```
 column_name |     data_type     | is_nullable |     column_default
-------------+-------------------+-------------+------------------------
 id          | integer           | NO          | nextval('users_id_seq')
 name        | character varying | NO          |
 email       | character varying | NO          |
 created_at  | timestamp         | YES         | CURRENT_TIMESTAMP
```

### View Indexes
```sql
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'users';
```

### View Constraints
```sql
SELECT conname, contype, pg_get_constraintdef(oid)
FROM pg_constraint
WHERE conrelid = 'users'::regclass;
```

---

## Data Validation Patterns

### Pattern 1: Verify Your Migration Ran

```sql
-- Check migration status
SELECT * FROM schema_migrations ORDER BY version DESC LIMIT 5;

-- Verify new column exists
SELECT column_name, data_type
FROM information_schema.columns
WHERE table_name = 'users' AND column_name = 'phone_number';

-- Verify data was backfilled
SELECT COUNT(*) FROM users WHERE phone_number IS NOT NULL;
```

### Pattern 2: Verify Your Query Returns Expected Data

```sql
-- You wrote this query for the API
SELECT 
    u.id,
    u.name,
    COUNT(o.id) as order_count,
    SUM(o.total) as total_spent
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
GROUP BY u.id, u.name
ORDER BY total_spent DESC
LIMIT 10;

-- Actually run it and verify the output
```

### Pattern 3: Test Before Destructive Operations

```sql
-- ALWAYS preview before DELETE or UPDATE
BEGIN;

-- See what would be affected
SELECT * FROM users WHERE last_login < '2023-01-01';

-- If it looks right, proceed
DELETE FROM users WHERE last_login < '2023-01-01';

-- Verify the result
SELECT COUNT(*) FROM users;

-- Happy? Commit. Scared? Rollback.
ROLLBACK;  -- or COMMIT;
```

---

## Performance Analysis

### Explain Your Query

```sql
EXPLAIN ANALYZE
SELECT u.*, COUNT(o.id)
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
WHERE u.created_at > '2024-01-01'
GROUP BY u.id;
```

Returns execution plan with actual timing:
```
 GroupAggregate  (cost=1234.56..2345.67 rows=1000 width=48) (actual time=12.345..23.456 rows=856 loops=1)
   Group Key: u.id
   ->  Sort  (cost=1234.56..1256.78 rows=8900 width=40) (actual time=11.234..12.345 rows=8900 loops=1)
         Sort Key: u.id
         Sort Method: quicksort  Memory: 1234kB
         ->  Hash Left Join  (cost=123.45..234.56 rows=8900 width=40) (actual time=1.234..5.678 rows=8900 loops=1)
               ...
 Planning Time: 0.234 ms
 Execution Time: 24.567 ms
```

**You can now identify bottlenecks with real numbers.**

### Check Table Statistics

```sql
SELECT 
    relname as table_name,
    n_live_tup as row_count,
    n_dead_tup as dead_rows,
    last_vacuum,
    last_analyze
FROM pg_stat_user_tables
WHERE schemaname = 'public';
```

### Find Missing Indexes

```sql
-- Queries that might need indexes
SELECT 
    schemaname, tablename, 
    seq_scan, seq_tup_read,
    idx_scan, idx_tup_fetch
FROM pg_stat_user_tables
WHERE seq_scan > idx_scan
ORDER BY seq_tup_read DESC;
```

---

## Data Integrity Checks

### Find Orphaned Records

```sql
-- Orders without users
SELECT o.* 
FROM orders o
LEFT JOIN users u ON u.id = o.user_id
WHERE u.id IS NULL;
```

### Find Duplicates

```sql
SELECT email, COUNT(*)
FROM users
GROUP BY email
HAVING COUNT(*) > 1;
```

### Check Foreign Key Violations

```sql
-- Find orders referencing non-existent products
SELECT o.id, o.product_id
FROM order_items o
LEFT JOIN products p ON p.id = o.product_id
WHERE p.id IS NULL;
```

---

## Safe Data Modification

### Always Use Transactions

```sql
BEGIN;

-- Your changes here
UPDATE users SET status = 'inactive' WHERE last_login < '2023-01-01';

-- Verify
SELECT COUNT(*) FROM users WHERE status = 'inactive';

-- Commit only if correct
COMMIT;  -- or ROLLBACK;
```

### Preview Before You Change

```sql
-- DON'T do this
DELETE FROM users WHERE some_condition;

-- DO this instead
SELECT * FROM users WHERE some_condition;  -- Preview first
-- Then, in a transaction, delete
```

---

## MySQL Specifics

### Show Tables
```sql
SHOW TABLES;
```

### Describe Table
```sql
DESCRIBE users;
-- or
SHOW CREATE TABLE users;
```

### Show Indexes
```sql
SHOW INDEX FROM users;
```

### Check Table Status
```sql
SHOW TABLE STATUS LIKE 'users';
```

---

## PostgreSQL Specifics

### List Tables
```sql
\dt
-- or
SELECT * FROM pg_tables WHERE schemaname = 'public';
```

### Describe Table
```sql
\d users
-- or use information_schema query above
```

### Current Connections
```sql
SELECT * FROM pg_stat_activity;
```

### Lock Information
```sql
SELECT * FROM pg_locks WHERE NOT granted;
```

---

## JSON Operations (PostgreSQL)

```sql
-- Query JSON column
SELECT data->>'name' as name
FROM events
WHERE data->>'type' = 'purchase';

-- JSON aggregation
SELECT jsonb_agg(row_to_json(u))
FROM users u
WHERE u.active = true;

-- Update JSON field
UPDATE users
SET settings = jsonb_set(settings, '{theme}', '"dark"')
WHERE id = 123;
```

---

## Common Debugging Queries

### Why Is This Slow?

```sql
-- Check query plan
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT ... your slow query ...;

-- Check if table needs vacuum
SELECT schemaname, relname, n_dead_tup, last_vacuum
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC;

-- Check for lock contention
SELECT * FROM pg_locks WHERE NOT granted;
```

### What's Using Space?

```sql
SELECT 
    relname as table_name,
    pg_size_pretty(pg_total_relation_size(relid)) as total_size
FROM pg_catalog.pg_statio_user_tables
ORDER BY pg_total_relation_size(relid) DESC;
```

### What's Happening Right Now?

```sql
SELECT pid, now() - pg_stat_activity.query_start AS duration, query, state
FROM pg_stat_activity
WHERE state != 'idle'
ORDER BY duration DESC;
```

---

## The Verification Pattern

When you write SQL for an application:

```sql
-- 1. Write the query
SELECT u.id, u.name, COUNT(o.id) as orders
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
GROUP BY u.id
ORDER BY orders DESC
LIMIT 10;

-- 2. Run it against real data
-- (Telepipe lets you do this!)

-- 3. Verify output matches expectations
-- - Right number of columns? ✓
-- - Right data types? ✓
-- - Right order? ✓
-- - Performance acceptable? ✓

-- 4. Now you can say "I tested this query. Here are the results."
```

---

## What You Can Now Say

**Before:**
> "This query should return the top 10 users by order count. The JOIN logic looks correct."

**After:**
> "I executed this query against the database. Results:
> - Returns 10 rows as expected
> - Highest user has 47 orders
> - Query executes in 23ms
> - Uses index scan on orders.user_id
> 
> Here's the actual output: [data]"

---

## The Power of Real Data

You've written countless queries. But until now, they were theoretical.

Now you can:
- Run the query
- See the actual results
- Measure actual performance
- Verify actual correctness

**Don't describe what the query should return. Show what it returns.**
