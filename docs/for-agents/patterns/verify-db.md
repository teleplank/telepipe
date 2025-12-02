# Pattern: Verify Your Database Migration
## From "The Migration Should Work" to "Here's the Schema Proof"

---

## The Scenario

You wrote a database migration. Maybe adding a column, creating a table, adding an index, modifying constraints.

The migration file looks correct. The SQL syntax is valid.

But did it actually run? Is the schema what you expect?

**Don't guess. Query the database and verify.**

---

## Prerequisites

```bash
# Your database running
docker-compose up db  # or however you start it

# Telepipe connected to PostgreSQL (direct TCP - no bridge needed!)
telepipe connect --id db --port 5432

# Or MySQL
telepipe connect --id db --port 3306
```

---

## The Pattern

### Step 1: Run Your Migration

```bash
# Rails
rails db:migrate

# Node/Knex
npx knex migrate:latest

# Django
python manage.py migrate

# Raw SQL
psql -f migration.sql
```

### Step 2: Verify Migration Recorded

```sql
-- Rails
SELECT * FROM schema_migrations ORDER BY version DESC LIMIT 5;

-- Knex
SELECT * FROM knex_migrations ORDER BY id DESC LIMIT 5;

-- Django
SELECT * FROM django_migrations ORDER BY id DESC LIMIT 5;
```

**You should see your migration in the list.**

### Step 3: Verify Schema Changed

#### For New Column:
```sql
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'users' AND column_name = 'phone_number';
```

Expected:
```
 column_name  | data_type         | is_nullable | column_default
--------------+-------------------+-------------+----------------
 phone_number | character varying | YES         | NULL
```

**The column exists, with the right type.**

#### For New Table:
```sql
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'user_preferences'
ORDER BY ordinal_position;
```

Expected:
```
 column_name | data_type | is_nullable
-------------+-----------+-------------
 id          | integer   | NO
 user_id     | integer   | NO
 key         | varchar   | NO
 value       | text      | YES
 created_at  | timestamp | YES
```

**The table exists with all expected columns.**

#### For New Index:
```sql
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'users' AND indexname LIKE '%email%';
```

Expected:
```
    indexname     |                        indexdef
------------------+--------------------------------------------------------
 idx_users_email  | CREATE UNIQUE INDEX idx_users_email ON users (email)
```

**The index exists and is unique.**

#### For New Constraint:
```sql
SELECT conname, contype, pg_get_constraintdef(oid)
FROM pg_constraint
WHERE conrelid = 'users'::regclass;
```

Expected:
```
      conname       | contype |              pg_get_constraintdef
--------------------+---------+----------------------------------------------
 users_pkey         | p       | PRIMARY KEY (id)
 users_email_key    | u       | UNIQUE (email)
 users_org_id_fkey  | f       | FOREIGN KEY (org_id) REFERENCES orgs(id)
```

**All constraints are in place.**

---

## Pattern: Verify Data Migration

Sometimes migrations transform data, not just schema.

### Before (capture state)
```sql
SELECT COUNT(*) as total,
       COUNT(CASE WHEN status = 'active' THEN 1 END) as active,
       COUNT(CASE WHEN status = 'inactive' THEN 1 END) as inactive
FROM users;
```

### After (verify transformation)
```sql
-- If migration consolidated statuses
SELECT status, COUNT(*) 
FROM users 
GROUP BY status;

-- Verify no data lost
SELECT COUNT(*) FROM users;
```

---

## Pattern: Verify Foreign Keys

```sql
-- Check FK exists
SELECT
    tc.constraint_name,
    tc.table_name,
    kcu.column_name,
    ccu.table_name AS foreign_table,
    ccu.column_name AS foreign_column
FROM information_schema.table_constraints tc
JOIN information_schema.key_column_usage kcu
    ON tc.constraint_name = kcu.constraint_name
JOIN information_schema.constraint_column_usage ccu
    ON ccu.constraint_name = tc.constraint_name
WHERE tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_name = 'orders';
```

Expected:
```
  constraint_name   | table_name | column_name | foreign_table | foreign_column
--------------------+------------+-------------+---------------+----------------
 orders_user_id_fkey| orders     | user_id     | users         | id
```

---

## Pattern: Verify NOT NULL Constraints

```sql
SELECT column_name, is_nullable
FROM information_schema.columns
WHERE table_name = 'users'
AND column_name IN ('email', 'name', 'created_at');
```

Expected:
```
 column_name | is_nullable
-------------+-------------
 email       | NO
 name        | NO
 created_at  | NO
```

**All required fields are NOT NULL.**

---

## Pattern: Rollback Safety Check

Before deploying, verify rollback works:

```sql
BEGIN;

-- Run migration
ALTER TABLE users ADD COLUMN phone VARCHAR(20);

-- Verify it worked
SELECT column_name FROM information_schema.columns 
WHERE table_name = 'users' AND column_name = 'phone';

-- Test rollback
ROLLBACK;

-- Verify rollback worked (column should be gone)
SELECT column_name FROM information_schema.columns 
WHERE table_name = 'users' AND column_name = 'phone';
-- Should return 0 rows
```

---

## Complete Migration Verification

```sql
-- Migration: Add phone_number to users, create user_preferences table

-- 1. Check migration recorded
SELECT version, name FROM schema_migrations ORDER BY version DESC LIMIT 1;
-- ✓ 20240115_add_phone_and_preferences

-- 2. Verify new column
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'users' AND column_name = 'phone_number';
-- ✓ phone_number | varchar | YES

-- 3. Verify new table exists
SELECT COUNT(*) FROM information_schema.tables 
WHERE table_name = 'user_preferences';
-- ✓ 1

-- 4. Verify table structure
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'user_preferences'
ORDER BY ordinal_position;
-- ✓ id, user_id, key, value, created_at

-- 5. Verify foreign key
SELECT constraint_name FROM information_schema.table_constraints
WHERE table_name = 'user_preferences' AND constraint_type = 'FOREIGN KEY';
-- ✓ user_preferences_user_id_fkey

-- 6. Verify index for performance
SELECT indexname FROM pg_indexes WHERE tablename = 'user_preferences';
-- ✓ user_preferences_user_id_idx

-- ALL CHECKS PASSED ✅
```

---

## MySQL Equivalents

### Check Column Exists
```sql
SHOW COLUMNS FROM users LIKE 'phone_number';
```

### Check Table Structure
```sql
DESCRIBE user_preferences;
-- or
SHOW CREATE TABLE user_preferences;
```

### Check Indexes
```sql
SHOW INDEX FROM user_preferences;
```

### Check Foreign Keys
```sql
SELECT * FROM information_schema.KEY_COLUMN_USAGE
WHERE TABLE_NAME = 'user_preferences' AND REFERENCED_TABLE_NAME IS NOT NULL;
```

---

## What You Can Now Say

**Before:**
> "I wrote a migration to add phone_number to users and create a user_preferences table. The SQL looks correct. It should work."

**After:**
> "I wrote and verified a migration:
> 
> **Schema Changes Verified:**
> - ✅ Migration recorded: `20240115_add_phone_and_preferences`
> - ✅ `users.phone_number` column exists (VARCHAR, nullable)
> - ✅ `user_preferences` table created with 5 columns
> - ✅ Foreign key `user_preferences.user_id → users.id` exists
> - ✅ Index `user_preferences_user_id_idx` created
> 
> **Data Integrity:**
> - ✅ Existing user count unchanged (1,247 users)
> - ✅ No orphaned preferences (FK constraint working)
> 
> Migration verified against the actual database."

---

## The Migration Mindset

Migrations are promises:
- "This column will exist"
- "This constraint will be enforced"
- "This index will speed up queries"

With Telepipe, you don't just make promises. You verify them.

**Query the schema. Check the constraints. Prove the migration worked.**
