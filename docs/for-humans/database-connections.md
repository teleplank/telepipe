# Database Connections

Guide to connecting Telepipe to databases. Direct TCP - no websocat needed!

---

## Why Databases Are Easy

Unlike Chrome CDP (which uses WebSockets), databases use **TCP directly**.

**No bridge needed:**
```bash
# This just works!
telepipe connect --id db --port 5432
```

---

## PostgreSQL

### Start PostgreSQL

**macOS (Homebrew):**
```bash
brew services start postgresql
```

**Linux (systemd):**
```bash
sudo systemctl start postgresql
```

**Docker:**
```bash
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres
```

### Verify It's Running
```bash
pg_isready
```

### Connect Telepipe
```bash
telepipe connect --id db --port 5432
```

### Test Connection
```bash
echo "SELECT version();" | telepipe exec --id db
```

### Common Operations

**List databases:**
```bash
echo "\l" | telepipe exec --id db
```

**List tables:**
```bash
echo "\dt" | telepipe exec --id db
```

**Describe table:**
```bash
echo "\d users" | telepipe exec --id db
```

**Query data:**
```bash
echo "SELECT * FROM users LIMIT 10;" | telepipe exec --id db
```

**Insert data:**
```bash
echo "INSERT INTO users (name, email) VALUES ('Test', 'test@example.com');" | telepipe exec --id db
```

**Check schema:**
```bash
echo "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'users';" | telepipe exec --id db
```

**Explain query:**
```bash
echo "EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%example%';" | telepipe exec --id db
```

### Cleanup
```bash
telepipe disconnect --id db
```

---

## MySQL

### Start MySQL

**macOS (Homebrew):**
```bash
brew services start mysql
```

**Linux (systemd):**
```bash
sudo systemctl start mysql
```

**Docker:**
```bash
docker run -d -p 3306:3306 -e MYSQL_ROOT_PASSWORD=password mysql
```

### Verify It's Running
```bash
mysqladmin ping -u root -p
```

### Connect Telepipe
```bash
telepipe connect --id mysql --port 3306
```

### Test Connection
```bash
echo "SELECT VERSION();" | telepipe exec --id mysql
```

### Common Operations

**Show databases:**
```bash
echo "SHOW DATABASES;" | telepipe exec --id mysql
```

**Use database:**
```bash
echo "USE mydb;" | telepipe exec --id mysql
```

**Show tables:**
```bash
echo "SHOW TABLES;" | telepipe exec --id mysql
```

**Describe table:**
```bash
echo "DESCRIBE users;" | telepipe exec --id mysql
```

**Query data:**
```bash
echo "SELECT * FROM users LIMIT 10;" | telepipe exec --id mysql
```

### Cleanup
```bash
telepipe disconnect --id mysql
```

---

## Redis

### Start Redis

**macOS (Homebrew):**
```bash
brew services start redis
```

**Linux (systemd):**
```bash
sudo systemctl start redis
```

**Docker:**
```bash
docker run -d -p 6379:6379 redis
```

### Verify It's Running
```bash
redis-cli ping
```

### Connect Telepipe
```bash
telepipe connect --id cache --port 6379
```

### Test Connection
```bash
echo "PING" | telepipe exec --id cache
```

**Should return:** `PONG`

### Common Operations

**Set value:**
```bash
echo "SET mykey hello" | telepipe exec --id cache
```

**Get value:**
```bash
echo "GET mykey" | telepipe exec --id cache
```

**List keys:**
```bash
echo "KEYS *" | telepipe exec --id cache
```

**Delete key:**
```bash
echo "DEL mykey" | telepipe exec --id cache
```

**Set with expiry:**
```bash
echo "SETEX tempkey 60 tempvalue" | telepipe exec --id cache
```

**Hash operations:**
```bash
echo "HSET user:1 name John email john@example.com" | telepipe exec --id cache
echo "HGETALL user:1" | telepipe exec --id cache
```

**List operations:**
```bash
echo "LPUSH queue task1" | telepipe exec --id cache
echo "RPOP queue" | telepipe exec --id cache
```

### Cleanup
```bash
telepipe disconnect --id cache
```

---

## MongoDB

**Note:** MongoDB uses a binary protocol. For best results, use `mongosh` through redirect mode.

### Start MongoDB
```bash
# Docker
docker run -d -p 27017:27017 mongo
```

### Connect via mongosh
```bash
telepipe redirect --id mongo -- mongosh --quiet
```

### Common Operations
```bash
echo "show dbs" | telepipe exec --id mongo
echo "use mydb" | telepipe exec --id mongo
echo "db.users.find().limit(10)" | telepipe exec --id mongo
```

### Cleanup
```bash
telepipe stop --id mongo
```

---

## SQLite

SQLite is a file-based database. Use redirect mode with the sqlite3 client.

### Connect
```bash
telepipe redirect --id sqlite -- sqlite3 mydb.sqlite
```

### Common Operations
```bash
echo ".tables" | telepipe exec --id sqlite
echo "SELECT * FROM users LIMIT 10;" | telepipe exec --id sqlite
```

### Cleanup
```bash
telepipe stop --id sqlite
```

---

## Authentication

### PostgreSQL with Password

**Option 1: Environment variable**
```bash
PGPASSWORD=mypassword telepipe redirect --id db -- psql -h localhost -U myuser -d mydb
```

**Option 2: .pgpass file**
```bash
# Create ~/.pgpass with: hostname:port:database:username:password
chmod 600 ~/.pgpass
telepipe redirect --id db -- psql -h localhost -U myuser -d mydb
```

### MySQL with Password

```bash
telepipe redirect --id mysql -- mysql -h localhost -u myuser -pmypassword mydb
```

### Redis with Password

```bash
echo "AUTH mypassword" | telepipe exec --id cache
echo "PING" | telepipe exec --id cache
```

---

## Remote Databases

By default, Telepipe connects to localhost. For remote databases:

```bash
telepipe connect --id remote-db --port 5432 --host db.example.com
```

**Security note:** Remote connections should use TLS. Consider SSH tunneling:

```bash
# SSH tunnel in background
ssh -L 5432:localhost:5432 user@db.example.com -N &

# Connect through tunnel
telepipe connect --id db --port 5432
```

---

## Multiple Databases

Connect to multiple databases simultaneously:

```bash
# PostgreSQL
telepipe connect --id postgres --port 5432

# MySQL
telepipe connect --id mysql --port 3306

# Redis
telepipe connect --id redis --port 6379

# Query each
echo "SELECT 1;" | telepipe exec --id postgres
echo "SELECT 1;" | telepipe exec --id mysql
echo "PING" | telepipe exec --id redis
```

---

## Troubleshooting

### Connection Refused

**Check service is running:**
```bash
# PostgreSQL
pg_isready

# MySQL
mysqladmin ping

# Redis
redis-cli ping
```

**Check port:**
```bash
lsof -i :<port>
```

### Permission Denied

**PostgreSQL:** Check `pg_hba.conf` allows connections from localhost.

**MySQL:** Verify user has privileges:
```sql
GRANT ALL ON *.* TO 'user'@'localhost';
```

### Wrong Port

**Check actual port:**

PostgreSQL:
```bash
cat /etc/postgresql/*/main/postgresql.conf | grep port
```

MySQL:
```bash
cat /etc/mysql/my.cnf | grep port
```

### Timeout

**Increase connection timeout in database config.**

**Or check firewall:**
```bash
sudo ufw status
```

---

## Best Practices

### Use Descriptive Session IDs
```bash
# Good
telepipe connect --id postgres-prod --port 5432
telepipe connect --id redis-cache --port 6379

# Bad
telepipe connect --id db1 --port 5432
telepipe connect --id db2 --port 6379
```

### Clean Up Sessions
```bash
# Always disconnect when done
telepipe disconnect --id db
```

### Use Transactions for Safety
```bash
echo "BEGIN; UPDATE users SET name='test'; ROLLBACK;" | telepipe exec --id db
```

### Check Session Status
```bash
telepipe info --id db
```

---

## Quick Reference

| Database | Port | Command |
|----------|------|---------|
| PostgreSQL | 5432 | `telepipe connect --id db --port 5432` |
| MySQL | 3306 | `telepipe connect --id mysql --port 3306` |
| Redis | 6379 | `telepipe connect --id cache --port 6379` |
| MongoDB | 27017 | `telepipe redirect --id mongo -- mongosh` |
| SQLite | - | `telepipe redirect --id sqlite -- sqlite3 db.sqlite` |

---

## Next Steps

- **Quick Start:** [quick-start.md](quick-start.md)
- **Chrome DevTools:** [chrome-devtools.md](chrome-devtools.md)
- **Troubleshooting:** [troubleshooting.md](troubleshooting.md)
- **Command Reference:** [../reference/commands.md](../reference/commands.md)
