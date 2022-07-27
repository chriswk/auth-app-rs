# Auth App

### Prerequisites

1. Rust
2. sqlx-cli (`cargo install sqlx-cli`)
3. Database
#### Database setup

The default database url in `.env.sample` expects the following sql to be run.

```sql
CREATE USER authapp WITH PASSWORD 'xccssddwwee2233';
CREATE DATABASE authapprs WITH OWNER authapp;
```

copy .env.sample to .env in your checked out folder.

Then run
```
sqlx migrate run
```


and enjoy your typechecked SQL queries.

### Development

* If you add new queries, remember to run `cargo sqlx prepare -- --lib` to update sqlx-data.json.
