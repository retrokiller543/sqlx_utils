# SQLx Utils

[![Crates.io](https://img.shields.io/crates/v/sqlx-utils.svg)](https://crates.io/crates/sqlx-utils)
[![Documentation](https://docs.rs/sqlx-utils/badge.svg)](https://docs.rs/sqlx-utils)
[![License](https://img.shields.io/crates/l/sqlx-utils.svg)](LICENSE)

SQLx Utils provides a comprehensive set of utilities for working with the [SQLx](https://github.com/launchbadge/sqlx) library in a structured and efficient way. It simplifies database interactions through type-safe filters, repository patterns, and powerful batch operations.

## Features

- **Type-safe SQL Filters**: Define reusable filter structs with SQL-like syntax
- **Repository Pattern**: Implement CRUD operations with minimal boilerplate
- **Transaction Support**: Execute operations within transactions for data consistency
- **Batch Operations**: Efficiently process large datasets in chunks
- **Connection Pool Management**: Simplified access to database pools
- **Multi-database Support**: Works with SQLx's supported database backends
- **Comprehensive Tracing**: Built-in instrumentation for debugging and monitoring

## Installation

Add SQLx Utils to your `Cargo.toml`:

```toml
[dependencies]
sqlx-utils = "1.1.1"
```

By default, the crate enables the `any` database feature. To use a specific database:

```toml
[dependencies]
sqlx-utils = { version = "1.1.0", default-features = false, features = ["postgres"] }
```

Available database features:
- `any` (default): Works with any SQLx-supported database
- `postgres`: PostgreSQL specific
- `mysql`: MySQL specific
- `sqlite`: SQLite specific

## Quick Start

### Setting up the Connection Pool

```rust
use sqlx_utils::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For any DB (with the `any` feature)
    install_default_drivers();

    // Initialize the pool
    let pool = PoolOptions::new()
        .max_connections(5)
        .connect("your_connection_string").await?;

    initialize_db_pool(pool);

    Ok(())
}
```

### Defining a Filter

```rust
use sqlx_utils::prelude::*;

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM users WHERE
        ?id = i64 AND
        ?name LIKE String AND
        ?age > i32
    }
}

// Usage:
let filter = UserFilter::new()
    .id(42)
    .name("Alice%");
```

### Creating a Repository

```rust
use sqlx_utils::prelude::*;

// 1. Define your model
struct User {
    id: i64,
    name: String,
    email: String,
}

impl Model for User {
    type Id = i64;

    fn get_id(&self) -> Option<Self::Id> {
        Some(self.id)
    }
}

// 2. Create a repository
repository! {
    pub UserRepo<User>;
}

// 3. Implement operations
repository_insert! {
    UserRepo<User>;

    insert_query(user) {
        sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
            .bind(&user.name)
            .bind(&user.email)
    }
}

repository_update! {
    UserRepo<User>;

    update_query(user) {
        sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
            .bind(&user.name)
            .bind(&user.email)
            .bind(user.id)
    }
}

repository_delete! {
    UserRepo<User>;

    delete_by_id_query(id) {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
    }
}
```

### Using the Repository

```rust
use sqlx_utils::prelude::*;

#[tokio::main]
async fn main() -> Result<(), sqlx_utils::Error> {
    // Create a new user
    let user = User {
        id: 0, // Will be assigned by DB
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Insert the user
    let user = USER_REPO.insert(user).await?;

    // Update the user
    let user = User {
        id: user.id,
        name: "Alice Smith".to_string(),
        email: user.email,
    };
    USER_REPO.update(user).await?;

    // Delete the user
    USER_REPO.delete_by_id(user.id).await?;

    Ok(())
}
```

### Working with Transactions

```rust
async fn transfer_funds(from: i64, to: i64, amount: f64) -> Result<(), sqlx_utils::Error> {
    ACCOUNT_REPO.with_transaction(|mut tx| async move {
        // Deduct from source account
        let from_account = ACCOUNT_REPO.get_by_id_with_executor(&mut tx, from).await?
            .ok_or_else(|| Error::Repository { message: "Source account not found".into() })?;
        
        let from_account = Account {
            balance: from_account.balance - amount,
            ..from_account
        };
        
        ACCOUNT_REPO.update_with_executor(&mut tx, from_account).await?;
        
        // Add to destination account
        let to_account = ACCOUNT_REPO.get_by_id_with_executor(&mut tx, to).await?
            .ok_or_else(|| Error::Repository { message: "Destination account not found".into() })?;
        
        let to_account = Account {
            balance: to_account.balance + amount,
            ..to_account
        };
        
        ACCOUNT_REPO.update_with_executor(&mut tx, to_account).await?;
        
        (Ok(()), tx)
    }).await
}
```

## Advanced Usage

### Filter Composition

```rust
let admin_filter = UserFilter::new().role("admin");
let active_filter = StatusFilter::new().status("active");

// Combine filters
let active_admins = admin_filter.and(active_filter);
```

### Batch Operations

```rust
// Process users in batches of 100
let users: Vec<User> = get_many_users();
USER_REPO.insert_batch::<100, _>(users).await?;
```

### Custom Repository Methods

```rust
impl UserRepo {
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        let filter = UserFilter::new().email(email);
        self.get_optional_by_filter(filter).await
    }
    
    pub async fn save_with_audit(&self, user: User, actor: &str) -> Result<User, Error> {
        self.with_transaction(|mut tx| async move {
            let result = self.save_with_executor(&mut tx, user).await?;
            
            // Log audit record
            let audit = AuditLog {
                entity_type: "user",
                entity_id: result.id.to_string(),
                actor: actor.to_string(),
                action: if result.get_id().is_none() { "create" } else { "update" }.to_string(),
                timestamp: chrono::Utc::now(),
            };
            
            AUDIT_REPO.insert_with_executor(&mut tx, audit).await?;
            
            (Ok(result), tx)
        }).await
    }
}
```

## Implementation Notes

- **Static Repositories**: The `repository!` macro creates a static instance using `LazyLock`, accessible via the uppercase name (e.g., `USER_REPO`).
- **Zero-sized Type (ZST) Repositories**: By adding `!zst` to the repository macro, you can create repositories with zero runtime cost.
- **Debugging Filters**: Enable the `filter_debug_impl` feature to automatically implement `Debug` for all generated filters.
- **Error Logging**: The `log_err` feature adds error logging to all repository operations.
- **Insert with IDs**: The `insert_duplicate` feature allows inserting records with existing IDs.

## Available Repository Traits

SQLx Utils provides several repository traits that can be implemented for your models:

- **Repository**: Base trait for all repositories
- **InsertableRepository**: For inserting new records
- **UpdatableRepository**: For updating existing records
- **SaveRepository**: For intelligently inserting or updating based on ID presence
- **DeleteRepository**: For removing records
- **SelectRepository**: For querying records
- **FilterRepository**: For querying with type-safe filters
- **TransactionRepository**: For working with transactions

## Development Status

SQLx Utils is in active development. The API may evolve between minor versions as I refine the interface based on user feedback.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.