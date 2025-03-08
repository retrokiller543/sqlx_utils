//! Trait for adding select capabilities to a repository

mod_def! {
    pub mod filter;
}

use crate::mod_def;
use crate::prelude::Database;
use crate::traits::{Model, Repository};
use crate::types::QueryAs;
use crate::utils::tracing_debug_log;
use sqlx::{Database as DatabaseTrait, Executor, FromRow};

/// Trait for repositories that can retrieve records from the database.
///
/// The `SelectRepository` trait extends the base [`Repository`] trait with methods
/// for querying and retrieving records. It defines a standard interface for fetching
/// both individual records by ID and collections of records.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository retrieves. Must implement the [`Model`] trait
///   and [`FromRow`] for the database's row type.
///
/// # Required Methods
///
/// Implementing repositories must define:
///
/// * [`get_all_query`](SelectRepository::get_all_query) - Creates a query to retrieve all records
/// * [`get_by_id_query`](SelectRepository::get_by_id_query) - Creates a query to retrieve a record by ID
///
/// # Provided Methods
///
/// These methods are automatically provided based on the required methods:
///
/// * [`get_all_with_executor`](SelectRepository::get_all_with_executor) - Execute the get_all query with a custom executor
/// * [`get_all`](SelectRepository::get_all) - Retrieve all records using the repository's pool
/// * [`get_by_id_with_executor`](SelectRepository::get_by_id_with_executor) - Execute the get_by_id query with a custom executor
/// * [`get_by_id`](SelectRepository::get_by_id) - Retrieve a record by ID using the repository's pool
///
/// # Examples
///
/// Basic implementation:
/// ```rust
/// use sqlx_utils::prelude::QueryAs;
/// use sqlx_utils::traits::{Model, Repository, SelectRepository};
/// # use sqlx_utils::types::Pool;
/// # #[derive(sqlx::FromRow)]
/// # struct User { id: i32, name: String }
/// # impl Model for User {
/// #     type Id = i32;
/// #     fn get_id(&self) -> Option<Self::Id> { Some(self.id) }
/// # }
/// # struct UserRepository { pool: Pool }
/// # impl Repository<User> for UserRepository {
/// #     fn pool(&self) -> &Pool { &self.pool }
/// # }
///
/// impl SelectRepository<User> for UserRepository {
///     fn get_all_query(&self) -> QueryAs<User> {
///         sqlx::query_as("SELECT * FROM users")
///     }
///
///     fn get_by_id_query(&self, id: impl Into<User::Id>) -> QueryAs<User> {
///         let id = id.into();
///         sqlx::query_as("SELECT * FROM users WHERE id = $1")
///             .bind(id)
///     }
/// }
///
/// // Usage
/// # async fn example(repo: &UserRepository) -> sqlx_utils::Result<()> {
/// // Get a single user
/// let user = repo.get_by_id(1).await?;
///
/// // Get all users
/// let all_users = repo.get_all().await?;
/// # Ok(())
/// # }
/// ```
///
/// # Implementation Notes
///
/// 1. When implementing this trait, you only need to define:
///    - [`get_all_query`](SelectRepository::get_all_query)
///    - [`get_by_id_query`](SelectRepository::get_by_id_query)
/// 2. The execution methods are provided automatically based on these query methods
/// 3. Consider implementing pagination for [`get_all`](SelectRepository::get_all) if the table may contain a large
///    number of records
/// 4. Use parameter binding to prevent SQL injection
/// 5. Consider caching strategies for frequently accessed data
/// 6. The trait supports using custom executors (like transactions) via the `_with_executor` methods
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `SelectRepository<{M}>` trait",
    label = "this type does not implement `SelectRepository` for model type `{M}`",
    message = "`{Self}` must implement `SelectRepository<{M}>` to query for `{M}` records",
    note = "Model `{M}` must implement `FromRow` for the database's row type. If you're seeing lifetime issues, ensure the model and repository properly handle the `'r` lifetime."
)]
pub trait SelectRepository<M: Model>: Repository<M>
where
    M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
{
    /// Creates a query to retrieve all records of this model type from the database.
    ///
    /// This is a required method that implementors must define. It creates a query
    /// that will select all records of the model type, without executing it.
    ///
    /// # Returns
    ///
    /// * [`QueryAs<M>`] - A prepared query that maps rows to the model type `M`
    ///
    /// # Warning
    ///
    /// Be cautious with this method on large tables as it could consume significant
    /// memory and impact database performance. Consider implementing pagination or
    /// adding WHERE clauses to limit the result set.
    ///
    /// # Implementation Example
    ///
    /// ```rust,ignore
    /// fn get_all_query(&self) -> QueryAs<User> {
    ///     sqlx::query_as("SELECT * FROM users")
    /// }
    /// ```
    ///
    /// ```rust,ignore
    /// fn get_all_query(&self) -> QueryAs<User> {
    ///     sqlx::query_as!(User, "SELECT * FROM users")
    /// }
    /// ```
    fn get_all_query(&self) -> QueryAs<M>;

    /// Creates a query to retrieve a single model instance by its ID.
    ///
    /// This is a required method that implementors must define. It creates a query
    /// that will select a record with the specified ID, without executing it.
    ///
    /// # Parameters
    ///
    /// * `id` - Any value that can be converted into the model's ID type
    ///
    /// # Returns
    ///
    /// * [`QueryAs<M>`] - A prepared query that maps rows to the model type `M`
    ///
    /// # Implementation Example
    ///
    /// ```rust,no_compile
    /// fn get_by_id_query(&self, id: impl Into<User::Id>) -> QueryAs<User> {
    ///     let id = id.into();
    ///     sqlx::query_as("SELECT * FROM users WHERE id = $1")
    ///         .bind(id)
    /// }
    /// ```
    ///
    /// ```rust,no_compile
    /// fn get_by_id_query(&self, id: impl Into<User::Id>) -> QueryAs<User> {
    ///     let id = id.into();
    ///     sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    /// }
    /// ```
    fn get_by_id_query(&self, id: impl Into<M::Id>) -> QueryAs<M>;

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "get_all",]
        /// Executes the `get_all` query with a custom executor.
        ///
        /// This method is automatically provided based on your implementation of
        /// [`get_all_query`](SelectRepository::get_all_query). It allows you to execute
        /// the query using a custom executor, such as a transaction.
        ///
        /// # Type Parameters
        ///
        /// * `E` - The executor type, such as a transaction or connection pool
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        ///
        /// # Returns
        ///
        /// * [`crate::Result<Vec<M>>`] - A Result containing a vector of all models if successful
        ///
        /// # Example
        ///
        /// ```rust,no_compile
        /// async fn get_users_in_transaction<'a>(&self, tx: &mut Transaction<'a, Database>) -> Result<Vec<User>> {
        ///     self.get_all_with_executor(&mut *tx).await
        /// }
        /// ```
        ///
        /// # Warning
        ///
        /// Be cautious with this method on large tables as it could consume significant
        /// memory and impact database performance. Consider implementing pagination instead.
        #[inline(always)]
        async fn get_all_with_executor<E>(
            &self,
            tx: E,
        ) -> crate::Result<Vec<M>>
        where
            E: for<'c> Executor<'c, Database = Database>,
        {
            self.get_all_query().fetch_all(tx).await.map_err(Into::into)
        }
    }

    /// Retrieves all records of this model type from the database.
    ///
    /// This method is automatically provided and simply calls [`get_all_with_executor`](SelectRepository::get_all_with_executor)
    /// with the repository's connection pool. It executes the query from
    /// [`get_all_query`](SelectRepository::get_all_query).
    ///
    /// # Returns
    ///
    /// * [`crate::Result<Vec<M>>`] - A Result containing a vector of all models if successful
    ///
    /// # Warning
    ///
    /// Be cautious with this method on large tables as it could consume significant
    /// memory and impact database performance. Consider implementing pagination instead.
    #[inline(always)]
    async fn get_all(&self) -> crate::Result<Vec<M>> {
        self.get_all_with_executor(self.pool()).await
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "get_by_id",]
        /// Executes the `get_by_id` query with a custom executor.
        ///
        /// This method is automatically provided based on your implementation of
        /// [`get_by_id_query`](SelectRepository::get_by_id_query). It allows you to execute
        /// the query using a custom executor, such as a transaction.
        ///
        /// # Type Parameters
        ///
        /// * `E` - The executor type, such as a transaction or connection pool
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `id` - Any value that can be converted into the model's ID type
        ///
        /// # Returns
        ///
        /// * [`crate::Result<Option<M>>`] - A Result containing either:
        ///   - `Some(model)` if a record was found
        ///   - `None` if no record exists with the given ID
        ///
        /// # Example
        ///
        /// ```rust,no_compile
        /// async fn get_user_in_transaction<'a>(&self, tx: &mut Transaction<'a, Database>, id: i32) -> Result<Option<User>> {
        ///     self.get_by_id_with_executor(&mut *tx, id).await
        /// }
        /// ```
        #[inline(always)]
        async fn get_by_id_with_executor<E>(
            &self,
            tx: E,
            id: impl Into<M::Id>
        ) -> crate::Result<Option<M>>
        where
            E: for<'c> Executor<'c, Database = Database>,
        {
            self.get_by_id_query(id).fetch_optional(tx).await.map_err(Into::into)
        }
    }

    /// Retrieves a single model instance by its ID.
    ///
    /// This method is automatically provided and simply calls [`get_by_id_with_executor`](SelectRepository::get_by_id_with_executor)
    /// with the repository's connection pool. It executes the query from
    /// [`get_by_id_query`](SelectRepository::get_by_id_query).
    ///
    /// # Parameters
    ///
    /// * `id` - Any value that can be converted into the model's ID type
    ///
    /// # Returns
    ///
    /// * [`crate::Result<Option<M>>`] - A Result containing either:
    ///   - `Some(model)` if a record was found
    ///   - `None` if no record exists with the given ID
    async fn get_by_id(&self, id: impl Into<M::Id>) -> crate::Result<Option<M>> {
        self.get_by_id_with_executor(self.pool(), id).await
    }
}
