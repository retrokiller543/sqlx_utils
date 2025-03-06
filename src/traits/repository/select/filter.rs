//! Filter related traits for repositories

use crate::traits::{Model, Repository, SqlFilter};
use crate::types::Database;
use cfg_if::cfg_if;
use sqlx::{Database as DatabaseTrait, Executor, FromRow, QueryBuilder};
use std::fmt::Debug;

macro_rules! filter_repository_methods {
    (skip($($ident:ident),*)  $($err:ident)? ; $($debug:ident)?) => {
        /// Retrieves all records matching the specified filter using a custom executor.
        ///
        /// This method applies the filter to a query and fetches all matching records using
        /// the provided executor. It's useful for including filter queries within transactions.
        ///
        /// # Type Parameters
        ///
        /// * `'a` - The lifetime of the filter and executor
        /// * `F` - The filter type, must implement [`SqlFilter`]
        /// * `E` - The executor type, must implement [`Executor`]
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `filter` - The filter to apply
        ///
        /// # Returns
        ///
        /// * [`crate::Result<Vec<M>>`] - A Result containing all matching models
        #[inline(always)]
        #[tracing::instrument(skip($($ident),*), level = "debug", parent = &Self::repository_span(), name = "get_by_filter", $($err, )?)]
        async fn get_all_by_any_filter_with_executor<'a, F, E>(
            &'a self,
            tx: E,
            filter: F,
        ) -> crate::Result<Vec<M>>
        where
            F: for<'c> SqlFilter<'c, Database> $(+ $debug)? + Send + 'a,
            E: for<'c> Executor<'c, Database = Database> + 'a,
        {
            Self::prepare_filter_query(filter).build_query_as().fetch_all(tx).await.map_err(Into::into)
        }

        /// Retrieves exactly one record matching the specified filter using a custom executor.
        ///
        /// This method applies the filter to a query and fetches exactly one matching record
        /// using the provided executor. It will error if no records match or if multiple records match.
        ///
        /// # Type Parameters
        ///
        /// * `'a` - The lifetime of the filter and executor
        /// * `F` - The filter type, must implement [`SqlFilter`]
        /// * `E` - The executor type, must implement [`Executor`]
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `filter` - The filter to apply
        ///
        /// # Returns
        ///
        /// * [`crate::Result<M>`] - A Result containing the matching model
        ///   - Error if no records match
        ///   - Error if multiple records match
        #[inline(always)]
        #[tracing::instrument(skip($($ident),*), level = "debug", parent = &Self::repository_span(), name = "get_by_filter", $($err, )?)]
        async fn get_one_by_any_filter_with_executor<'a, F, E>(
            &'a self,
            tx: E,
            filter: F,
        ) -> crate::Result<M>
        where
            F: for<'c> SqlFilter<'c, Database> $(+ $debug)? + Send + 'a,
            E: for<'c> Executor<'c, Database = Database> + 'a,
        {
            Self::prepare_filter_query(filter).build_query_as().fetch_one(tx).await.map_err(Into::into)
        }

        /// Retrieves an optional record matching the specified filter using a custom executor.
        ///
        /// This method applies the filter to a query and fetches at most one matching record
        /// using the provided executor. It returns `None` if no records match, and errors if
        /// multiple records match.
        ///
        /// # Type Parameters
        ///
        /// * `'a` - The lifetime of the filter and executor
        /// * `F` - The filter type, must implement [`SqlFilter`]
        /// * `E` - The executor type, must implement [`Executor`]
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `filter` - The filter to apply
        ///
        /// # Returns
        ///
        /// * [`crate::Result<Option<M>>`] - A Result containing:
        ///   - `None` if no records match
        ///   - `Some(model)` if exactly one record matches
        ///   - Error if multiple records match
        #[inline(always)]
        #[tracing::instrument(skip($($ident),*), level = "debug", parent = &Self::repository_span(), name = "get_by_filter", $($err, )?)]
        async fn get_optional_by_any_filter_with_executor<'a, F, E>(
            &'a self,
            tx: E,
            filter: F,
        ) -> crate::Result<Option<M>>
        where
            F: for<'c> SqlFilter<'c, Database> $(+ $debug)? + Send + 'a,
            E: for<'c> Executor<'c, Database = Database> + 'a,
        {
            Self::prepare_filter_query(filter).build_query_as().fetch_optional(tx).await.map_err(Into::into)
        }

        /// Retrieves all records matching the specified filter using the repository's connection pool.
        ///
        /// This is a convenience method that uses the repository's own connection pool
        /// as the executor for filter-based queries.
        ///
        /// # Type Parameters
        ///
        /// * `'a` - The lifetime of the filter
        /// * `F` - The filter type, must implement [`SqlFilter`]
        ///
        /// # Parameters
        ///
        /// * `filter` - The filter to apply
        ///
        /// # Returns
        ///
        /// * [`crate::Result<Vec<M>>`] - A Result containing all matching models
        #[inline(always)]
        async fn get_all_by_any_filter<'a, F>(
            &'a self,
            filter: F,
        ) -> crate::Result<Vec<M>>
        where
            F: for<'c> SqlFilter<'c, Database> $(+ $debug)? + Send + 'a,
        {
                let pool = self.pool();
                self.get_all_by_any_filter_with_executor(pool, filter).await
        }

        /// Retrieves exactly one record matching the specified filter using the repository's connection pool.
        ///
        /// This is a convenience method that uses the repository's own connection pool
        /// as the executor for filter-based queries.
        ///
        /// # Type Parameters
        ///
        /// * `'a` - The lifetime of the filter
        /// * `F` - The filter type, must implement [`SqlFilter`]
        ///
        /// # Parameters
        ///
        /// * `filter` - The filter to apply
        ///
        /// # Returns
        ///
        /// * [`crate::Result<M>`] - A Result containing the matching model
        ///   - Error if no records match
        ///   - Error if multiple records match
        #[inline(always)]
        async fn get_one_by_any_filter<'a, F>(
            &'a self,
            filter: F,
        ) -> crate::Result<M>
        where
            F: for<'c> SqlFilter<'c, Database> $(+ $debug)? + Send + 'a,
        {
                let pool = self.pool();
                self.get_one_by_any_filter_with_executor(pool, filter).await
        }

        /// Retrieves an optional record matching the specified filter using the repository's connection pool.
        ///
        /// This is a convenience method that uses the repository's own connection pool
        /// as the executor for filter-based queries.
        ///
        /// # Type Parameters
        ///
        /// * `'a` - The lifetime of the filter
        /// * `F` - The filter type, must implement [`SqlFilter`]
        ///
        /// # Parameters
        ///
        /// * `filter` - The filter to apply
        ///
        /// # Returns
        ///
        /// * [`crate::Result<Option<M>>`] - A Result containing:
        ///   - `None` if no records match
        ///   - `Some(model)` if exactly one record matches
        ///   - Error if multiple records match
        #[inline(always)]
        async fn get_optional_by_any_filter<'a, F>(
            &'a self,
            filter: F,
        ) -> crate::Result<Option<M>>
        where
            F: for<'c> SqlFilter<'c, Database> $(+ $debug)? + Send + 'a,
        {
                let pool = self.pool();
                self.get_optional_by_any_filter_with_executor(pool, filter).await
        }
    };
}

macro_rules! filter_repository_ext {
    ($($debug:ident)?) => {
        /// Extension trait providing type-specific filter methods for repositories.
        ///
        /// The `FilterRepositoryExt` trait complements the [`FilterRepository`] trait by adding
        /// methods optimized for specific filter types. While [`FilterRepository`] provides
        /// generic filter capabilities with methods like [`get_all_by_any_filter`](FilterRepository::get_all_by_any_filter),
        /// this trait adds type-specific methods (like [`get_all_by_filter`](FilterRepositoryExt::get_all_by_filter))
        /// that allow for better type inference and a more ergonomic API when using known filter types.
        ///
        /// This trait is automatically implemented for any type that implements [`FilterRepository`],
        /// so no manual implementation is needed.
        ///
        /// # Type Parameters
        ///
        /// * `M` - The model type that this repository filters. Must implement the [`Model`] trait
        ///   and [`FromRow`] for the database's row type.
        /// * `Filter` - The specific filter type used with this repository extension.
        ///
        /// # Examples
        ///
        /// Using type-specific filter methods:
        /// ```rust
        /// # use sqlx_utils::traits::{Model, Repository, FilterRepository, FilterRepositoryExt, SqlFilter};
        /// # use sqlx_utils::types::{Pool, Database};
        /// # use sqlx_utils::sql_filter;
        /// # use sqlx::{FromRow, QueryBuilder};
        /// # use std::fmt::Debug;
        /// # struct User { id: i32, name: String, email: String, age: i32 }
        /// # impl Model for User {
        /// #     type Id = i32;
        /// #     fn get_id(&self) -> Option<Self::Id> { Some(self.id) }
        /// # }
        /// # impl FromRow<'_, sqlx::any::AnyRow> for User {
        /// #     fn from_row(_: &sqlx::any::AnyRow) -> Result<Self, sqlx::Error> {
        /// #         unimplemented!()
        /// #     }
        /// # }
        /// # struct UserRepository { pool: Pool }
        /// # impl Repository<User> for UserRepository {
        /// #     fn pool(&self) -> &Pool { &self.pool }
        /// # }
        /// # impl FilterRepository<User> for UserRepository {
        /// #     fn filter_query_builder<'args>() -> QueryBuilder<'args, Database> {
        /// #         QueryBuilder::new("SELECT * FROM users WHERE ")
        /// #     }
        /// # }
        ///
        /// // Define a specific filter type
        /// sql_filter! {
        ///     #[derive(Clone)]
        ///     pub struct UserNameFilter {
        ///         SELECT * FROM users WHERE
        ///         name LIKE String
        ///     }
        /// }
        ///
        /// // Using the type-specific extension methods
        /// # async fn example(repo: &UserRepository) -> sqlx_utils::Result<()> {
        /// let name_filter = UserNameFilter::new("Alice%");
        ///
        /// // These methods know exactly what filter type to expect
        /// let all_users = repo.get_all_by_filter(name_filter.clone()).await?;
        /// let one_user = repo.get_one_by_filter(name_filter.clone()).await?;
        /// let optional_user = repo.get_optional_by_filter(name_filter).await?;
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// # Feature Flags
        ///
        /// This trait can be configured with feature flags:
        ///
        /// * `filter_debug_impl` - Adds a [`Debug`] bound to the `Filter` type parameter,
        ///   enabling better debug output but requiring filter types to implement [`Debug`]
        ///
        /// # Available Methods
        ///
        /// The trait provides several type-specific ways to fetch filtered records:
        ///
        /// 1. Multiple Records:
        ///    - [`get_all_by_filter`](FilterRepositoryExt::get_all_by_filter) - Get all matching records
        ///    - [`get_all_by_filter_with_executor`](FilterRepositoryExt::get_all_by_filter_with_executor) - Same, but with a custom executor
        ///
        /// 2. Single Record:
        ///    - [`get_one_by_filter`](FilterRepositoryExt::get_one_by_filter) - Get exactly one record (errors if none or multiple)
        ///    - [`get_one_by_filter_with_executor`](FilterRepositoryExt::get_one_by_filter_with_executor) - Same, but with a custom executor
        ///
        /// 3. Optional Record:
        ///    - [`get_optional_by_filter`](FilterRepositoryExt::get_optional_by_filter) - Get a record if it exists
        ///    - [`get_optional_by_filter_with_executor`](FilterRepositoryExt::get_optional_by_filter_with_executor) - Same, but with a custom executor
        #[diagnostic::on_unimplemented(
            message = "Type `{Self}` cannot use `FilterRepositoryExt<{M}, {Filter}>` because it does not implement `FilterRepository<{M}>`",
            label = "this type needs to implement `FilterRepository<{M}>` first",
            note = "Make sure your repository implements `FilterRepository<{M}>` and that `{Filter}` implements `SqlFilter<'args, Database>`",
            note = "The FilterRepositoryExt trait is automatically implemented for any type that implements FilterRepository, so you just need to implement FilterRepository for your repository type."
        )]
        pub trait FilterRepositoryExt<M, Filter>: FilterRepository<M>
        where
            M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
            Filter: for<'args> SqlFilter<'args, Database> $(+ $debug)? + Send,
            Self: Sync,
        {
            /// Retrieves all records matching the specified filter using a custom executor.
            ///
            /// This method is similar to [`get_all_by_any_filter_with_executor`](FilterRepository::get_all_by_any_filter_with_executor)
            /// but with a specific filter type, allowing for better type inference.
            ///
            /// # Parameters
            ///
            /// * `tx` - The executor to use for the query
            /// * `filter` - The typed filter to apply
            ///
            /// # Returns
            ///
            /// * [`crate::Result<Vec<M>>`] - A Result containing all matching models
            #[inline(always)]
            async fn get_all_by_filter_with_executor<E>(
                &self,
                tx: E,
                filter: Filter,
            ) -> crate::Result<Vec<M>>
            where
                E: for<'c> Executor<'c, Database = Database>,
            {
                self.get_all_by_any_filter_with_executor(tx, filter).await
            }

            /// Retrieves all records matching the specified filter using the repository's connection pool.
            ///
            /// This method is similar to [`get_all_by_any_filter`](FilterRepository::get_all_by_any_filter)
            /// but with a specific filter type, allowing for better type inference.
            ///
            /// # Parameters
            ///
            /// * `filter` - The typed filter to apply
            ///
            /// # Returns
            ///
            /// * [`crate::Result<Vec<M>>`] - A Result containing all matching models
            #[inline(always)]
            async fn get_all_by_filter(
                &self,
                filter: Filter,
            ) -> crate::Result<Vec<M>>
            {
                let pool = self.pool();
                self.get_all_by_filter_with_executor(pool, filter).await
            }

            /// Retrieves exactly one record matching the specified filter using a custom executor.
            ///
            /// This method is similar to [`get_one_by_any_filter_with_executor`](FilterRepository::get_one_by_any_filter_with_executor)
            /// but with a specific filter type, allowing for better type inference.
            ///
            /// # Parameters
            ///
            /// * `tx` - The executor to use for the query
            /// * `filter` - The typed filter to apply
            ///
            /// # Returns
            ///
            /// * [`crate::Result<M>`] - A Result containing the matching model
            ///   - Error if no records match
            ///   - Error if multiple records match
            #[inline(always)]
            async fn get_one_by_filter_with_executor<E>(
                &self,
                tx: E,
                filter: Filter,
            ) -> crate::Result<M>
            where
                E: for<'c> Executor<'c, Database = Database>,
            {
                self.get_one_by_any_filter_with_executor(tx, filter).await
            }

            /// Retrieves exactly one record matching the specified filter using the repository's connection pool.
            ///
            /// This method is similar to [`get_one_by_any_filter`](FilterRepository::get_one_by_any_filter)
            /// but with a specific filter type, allowing for better type inference.
            ///
            /// # Parameters
            ///
            /// * `filter` - The typed filter to apply
            ///
            /// # Returns
            ///
            /// * [`crate::Result<M>`] - A Result containing the matching model
            ///   - Error if no records match
            ///   - Error if multiple records match
            #[inline(always)]
            async fn get_one_by_filter(
                &self,
                filter: Filter,
            ) -> crate::Result<M>
            {
                let pool = self.pool();
                self.get_one_by_filter_with_executor(pool, filter).await
            }

            /// Retrieves an optional record matching the specified filter using a custom executor.
            ///
            /// This method is similar to [`get_optional_by_any_filter_with_executor`](FilterRepository::get_optional_by_any_filter_with_executor)
            /// but with a specific filter type, allowing for better type inference.
            ///
            /// # Parameters
            ///
            /// * `tx` - The executor to use for the query
            /// * `filter` - The typed filter to apply
            ///
            /// # Returns
            ///
            /// * [`crate::Result<Option<M>>`] - A Result containing:
            ///   - `None` if no records match
            ///   - `Some(model)` if exactly one record matches
            ///   - Error if multiple records match
            #[inline(always)]
            async fn get_optional_by_filter_with_executor<E>(
                &self,
                tx: E,
                filter: Filter,
            ) -> crate::Result<Option<M>>
            where
                E: for<'c> Executor<'c, Database = Database>,
            {
                self.get_optional_by_any_filter_with_executor(tx, filter).await
            }

            /// Retrieves an optional record matching the specified filter using the repository's connection pool.
            ///
            /// This method is similar to [`get_optional_by_any_filter`](FilterRepository::get_optional_by_any_filter)
            /// but with a specific filter type, allowing for better type inference.
            ///
            /// # Parameters
            ///
            /// * `filter` - The typed filter to apply
            ///
            /// # Returns
            ///
            /// * [`crate::Result<Option<M>>`] - A Result containing:
            ///   - `None` if no records match
            ///   - `Some(model)` if exactly one record matches
            ///   - Error if multiple records match
            #[inline(always)]
            async fn get_optional_by_filter(
                &self,
                filter: Filter,
            ) -> crate::Result<Option<M>>
            {
                let pool = self.pool();
                self.get_optional_by_filter_with_executor(pool, filter).await
            }
        }

        impl<M, Filter, T> FilterRepositoryExt<M, Filter> for T
        where
            T: FilterRepository<M>,
            M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
            Filter: for<'args> SqlFilter<'args, Database> $(+ $debug)? + Send,
            Self: Sync,
        {}
    };
}

/// Trait for repositories that support complex filtering of records.
///
/// The `FilterRepository` trait extends the base [`Repository`] trait with methods for
/// querying records using composable, type-safe filter conditions. It enables powerful
/// and reusable filtering logic that can be combined to create complex queries while
/// maintaining type safety and preventing SQL injection.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository filters. Must implement the [`Model`] trait
///   and [`FromRow`] for the database's row type.
///
/// # Examples
///
/// Basic implementation using the sql_filter macro:
/// ```rust
/// # use sqlx_utils::traits::{Model, Repository, FilterRepository, SqlFilter};
/// # use sqlx_utils::types::{Pool, Database};
/// # use sqlx_utils::sql_filter;
/// # use sqlx::{FromRow, QueryBuilder};
/// # struct User { id: i32, name: String, email: String, age: i32 }
/// # impl Model for User {
/// #     type Id = i32;
/// #     fn get_id(&self) -> Option<Self::Id> { Some(self.id) }
/// # }
/// # impl FromRow<'_, sqlx::any::AnyRow> for User {
/// #     fn from_row(_: &sqlx::any::AnyRow) -> Result<Self, sqlx::Error> {
/// #         unimplemented!()
/// #     }
/// # }
/// # struct UserRepository { pool: Pool }
/// # impl Repository<User> for UserRepository {
/// #     fn pool(&self) -> &Pool { &self.pool }
/// # }
///
/// // Define a filter that can be reused across queries
/// sql_filter! {
///     #[derive(Clone)]
///     pub struct UserAgeFilter {
///         SELECT * FROM users WHERE
///         ?min_age as min_age >= i32 AND
///         ?max_age as max_age <= i32
///     }
/// }
///
/// impl FilterRepository<User> for UserRepository {
///     fn filter_query_builder<'args>() -> QueryBuilder<'args, Database> {
///         QueryBuilder::new("SELECT * FROM users WHERE ")
///     }
/// }
///
/// // Usage
/// # async fn example(repo: &UserRepository) -> sqlx_utils::Result<()> {
/// // Filter users by age range
/// let filter = UserAgeFilter::new()
///     .min_age(18)
///     .max_age(65);
///
/// // Get all matching records
/// let filtered_users = repo.get_all_by_any_filter(filter.clone()).await?;
///
/// // Get a single record (will error if no matches or multiple matches)
/// let single_user = repo.get_one_by_any_filter(filter.clone()).await?;
///
/// // Get an optional record (useful when you're not sure if it exists)
/// let maybe_user = repo.get_optional_by_any_filter(filter).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Available Methods
///
/// The trait provides several ways to fetch filtered records:
///
/// 1. Multiple Records:
///    - [`get_all_by_any_filter`](FilterRepository::get_all_by_any_filter) - Get all matching records
///    - [`get_all_by_any_filter_with_executor`](FilterRepository::get_all_by_any_filter_with_executor) - Same, but with a custom executor
///
/// 2. Single Record:
///    - [`get_one_by_any_filter`](FilterRepository::get_one_by_any_filter) - Get exactly one record (errors if none or multiple)
///    - [`get_one_by_any_filter_with_executor`](FilterRepository::get_one_by_any_filter_with_executor) - Same, but with a custom executor
///
/// 3. Optional Record:
///    - [`get_optional_by_any_filter`](FilterRepository::get_optional_by_any_filter) - Get a record if it exists
///    - [`get_optional_by_any_filter_with_executor`](FilterRepository::get_optional_by_any_filter_with_executor) - Same, but with a custom executor
///
/// # Implementation Notes
///
/// 1. Required method: [`filter_query_builder`](FilterRepository::filter_query_builder) - Creates a query builder for filter-based queries
/// 2. Query is built using [`prepare_filter_query`](FilterRepository::prepare_filter_query) and if anything is needed after the `WHERE` clause you need to override the
///    [`post_filter_query_builder`](FilterRepository::post_filter_query_builder) this allows you to modify query or even scrap it and construct a new one.
/// 3. All methods are instrumented with tracing for debugging and monitoring
/// 4. Feature flags control the debug implementation and error logging behavior:
///    - `filter_debug_impl` - Adds a `Debug` bound to filter types
///    - `log_err` - Enables error logging in instrumentation
/// 5. Works with filters created using the `sql_filter!` macro or custom `SqlFilter` implementations
/// 6. All queries are properly parameterized to prevent SQL injection
#[diagnostic::on_unimplemented(
    message = "`{Self}` must implement `FilterRepository<{M}>` to filter `{M}` records with SqlFilter",
    label = "this type does not implement `FilterRepository` for model type `{M}`",
    note = "Type `{Self}` does not implement the `FilterRepository<{M}>` trait",
    note = "Model `{M}` must implement `FromRow` for the database's row type. If you're seeing lifetime issues, ensure the model and repository properly handle the `'r` lifetime."
)]
pub trait FilterRepository<M>: Repository<M>
where
    M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
    Self: Sync,
{
    /// Creates a query builder for filter-based queries.
    ///
    /// This method should return a query builder initialized with the appropriate
    /// SELECT statement prefix. It forms the foundation for all filter-based queries
    /// in the repository.
    ///
    /// NOTE: You must not include a WHERE clause in the query
    ///
    /// # Type Parameters
    ///
    /// * `'args` - The lifetime for query arguments
    ///
    /// # Returns
    ///
    /// * [`QueryBuilder`] - A new query builder configured for this repository
    fn filter_query_builder<'args>() -> QueryBuilder<'args, Database>;

    /// Builds the Query and applies the given filter only if the filter has defined that
    /// it should be applied, it will also append the start of the `WHERE` clause.
    #[inline]
    fn prepare_filter_query<'args>(filter: impl SqlFilter<'args>) -> QueryBuilder<'args, Database> {
        let mut builder = Self::filter_query_builder();

        if filter.should_apply_filter() {
            builder.push("WHERE ");

            filter.apply_filter(&mut builder);
        }

        let builder = Self::post_filter_query(builder);

        builder
    }

    /// If you need anything to be after the WHERE clause in the query you will need to override this
    /// method to add it.
    #[inline(always)]
    fn post_filter_query(builder: QueryBuilder<Database>) -> QueryBuilder<Database> {
        builder
    }

    cfg_if! {
        if #[cfg(all(feature = "filter_debug_impl", feature = "log_err"))] {
            filter_repository_methods! {
                skip(self, tx) err; Debug
            }
        } else if #[cfg(feature = "log_err")] {
            filter_repository_methods! {
                skip(self, tx, filter) err;
            }
        } else if #[cfg(feature = "filter_debug_impl")] {
            filter_repository_methods! {
                skip(self, tx); Debug
            }
        } else {
            filter_repository_methods! {
                skip(self, tx, filter);
            }
        }
    }
}

#[cfg(feature = "filter_debug_impl")]
filter_repository_ext! {Debug}

#[cfg(not(feature = "filter_debug_impl"))]
filter_repository_ext! {}
