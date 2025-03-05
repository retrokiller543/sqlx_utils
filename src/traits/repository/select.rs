use crate::traits::{Model, Repository, SqlFilter};

pub trait SelectRepository<'a, M: Model>: Repository<M> {
    type Filter: SqlFilter<'a>;

    /// Gets by predefined filter
    async fn get_by_filter(&self, filter: Self::Filter) -> crate::Result<Vec<M>> {
        <Self as SelectRepository<'_, M>>::get_by_any_filter(self, filter).await
    }
    
    /// Retrieves all records of this model type from the database.
    ///
    /// By default, this method is unimplemented and will panic if called. Repositories
    /// should override this method when they need to support retrieving all records.
    /// Consider implementing pagination or limiting the result set size for large tables.
    ///
    /// # Returns
    ///
    /// * [`crate::Result<Vec<M>>`] - A Result containing a vector of all models if successful
    ///
    /// # Warning
    ///
    /// Be cautious with this method on large tables as it could consume significant
    /// memory and impact database performance. Consider implementing pagination instead.
    async fn get_all(&self) -> crate::Result<Vec<M>>;

    /// Gets by any filter
    async fn get_by_any_filter(&self, filter: impl SqlFilter<'_>) -> crate::Result<Vec<M>>;

    /// Retrieves a single model instance by its ID.
    ///
    /// By default, this method is unimplemented. When implemented, it should efficiently
    /// fetch exactly one record matching the provided ID. The method accepts any type
    /// that can be converted into the model's ID type for flexibility.
    ///
    /// # Parameters
    ///
    /// * `id` - Any value that can be converted into the model's ID type
    ///
    /// # Returns
    ///
    /// * [`crate::Result<Option<M>>`] - A Result containing either:
    ///   - Some(model) if a record was found
    ///   - None if no record exists with the given ID
    async fn get_by_id(&self, id: impl Into<M::Id>) -> crate::Result<Option<M>>;
}