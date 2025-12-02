# SQL Query Optimization Agent Prompt
## Your Mission: Optimize Queries with Proof

---

## Context

You are connected to a PostgreSQL database (`ecommerce_demo`) via Telepipe. This database has:
- **100,000 users**
- **500,000 orders**
- **10,000 products**

**The database has NO INDEXES** (except primary keys). This is intentional.

Your job: Find slow queries, optimize them, and **prove** your improvements.

---

## Your Capabilities

You can execute SQL directly through Telepipe:

```bash
telepipe exec --id db
```

Then type SQL commands. Use `\q` to exit.

---

## Your Mission

### Phase 1: Discovery

Run `EXPLAIN ANALYZE` on common queries to find bottlenecks:

```sql
-- Test email search
EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%@gmail.com';

-- Test order history
EXPLAIN ANALYZE SELECT * FROM orders
WHERE user_id = 12345
ORDER BY created_at DESC
LIMIT 10;

-- Test product search
EXPLAIN ANALYZE SELECT * FROM products WHERE name ILIKE '%laptop%';
```

**Look for:**
- `Seq Scan` = reading entire table (BAD)
- High `actual time` values (BAD)
- `Rows Removed by Filter` = wasted work (BAD)

---

### Phase 2: Analysis

For each slow query, identify:

1. **What column is being filtered?**
2. **What type of comparison?** (=, LIKE, ILIKE, range)
3. **What index type is appropriate?**

**Index Type Guide:**

| Pattern | Index Type | Example |
|---------|------------|---------|
| `column = value` | B-tree | `user_id = 12345` |
| `column LIKE 'prefix%'` | B-tree + text_pattern_ops | `email LIKE 'user%'` |
| `column LIKE '%suffix'` | B-tree + text_pattern_ops | `email LIKE '%@gmail.com'` |
| Full-text search | GIN + tsvector | `to_tsvector @@ to_tsquery` |
| `ORDER BY column` | B-tree | `ORDER BY created_at` |

---

### Phase 3: Optimization

Create the appropriate indexes:

```sql
-- For email pattern matching
CREATE INDEX idx_users_email ON users USING btree(email text_pattern_ops);

-- For user_id lookups
CREATE INDEX idx_orders_user_id ON orders(user_id);

-- For date ordering
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);

-- For full-text product search
CREATE INDEX idx_products_name_gin ON products USING gin(to_tsvector('english', name));
```

---

### Phase 4: Verification

**This is the critical step.** Re-run `EXPLAIN ANALYZE` and compare:

```sql
-- After index creation
EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%@gmail.com';
```

**Look for:**
- `Index Scan` or `Bitmap Index Scan` = using index (GOOD)
- Lower `actual time` values (GOOD)
- No `Rows Removed by Filter` (GOOD)

---

## Expected Results

| Query | Before | After | Improvement |
|-------|--------|-------|-------------|
| Email search | Seq Scan, ~5000ms | Index Scan, ~50ms | **100x** |
| Order history | Seq Scan, ~8000ms | Index Scan, ~0.1ms | **67,000x** |
| Product search | Seq Scan, ~2000ms | Index Scan, ~45ms | **54x** |

---

## Reporting Format

When you complete optimization, report like this:

```
## Optimization Report

### Query 1: Email Search
- **Before:** Seq Scan on users (actual time=0.028..5234.567)
- **Index Created:** CREATE INDEX idx_users_email ON users USING btree(email text_pattern_ops)
- **After:** Bitmap Index Scan (actual time=0.987..52.456)
- **Improvement:** 100x faster (5234ms → 52ms)

### Query 2: Order History
...

### Summary
| Query | Before | After | Improvement |
|-------|--------|-------|-------------|
| Email | 5234ms | 52ms | 100x |
| Orders | 8123ms | 0.12ms | 67,000x |
| Products | 2456ms | 45ms | 54x |

All improvements verified with EXPLAIN ANALYZE.
```

---

## Important Notes

1. **Always EXPLAIN ANALYZE before AND after** - No exceptions
2. **Report actual times** - Not estimated costs
3. **Note scan type changes** - Seq Scan → Index Scan
4. **Include the index DDL** - Show exactly what you created
5. **Calculate improvement factor** - Before / After = Nx faster

---

## Anti-Patterns to Avoid

**DON'T:**
- Create indexes without measuring first
- Report "should be faster" without proof
- Use estimated cost instead of actual time
- Skip the before measurement
- Assume an index will help without verifying

**DO:**
- Measure → Optimize → Measure → Report
- Use `EXPLAIN ANALYZE` (not just `EXPLAIN`)
- Report actual execution times
- Show before/after comparison
- Calculate and report improvement factor

---

## Full-Text Search Note

For product name search, you need to:

1. **Create GIN index on tsvector:**
```sql
CREATE INDEX idx_products_name_gin ON products
USING gin(to_tsvector('english', name));
```

2. **Change the query syntax:**
```sql
-- Instead of:
SELECT * FROM products WHERE name ILIKE '%laptop%';

-- Use:
SELECT * FROM products
WHERE to_tsvector('english', name) @@ to_tsquery('laptop');
```

This enables true full-text search with the GIN index.

---

## Telepipe Commands Reference

```bash
# Start SQL session
telepipe exec --id db

# Check connection
telepipe info --id db

# Disconnect when done
telepipe disconnect --id db
```

---

## Success Criteria

You succeed when you:

1. Run EXPLAIN ANALYZE on at least 3 queries
2. Identify Seq Scan bottlenecks
3. Create appropriate indexes
4. Verify Index Scan usage after
5. Report actual time improvements
6. Calculate improvement factors

**The goal is PROOF, not hope.**

---

*Now go optimize. Measure everything. Prove your improvements.*
