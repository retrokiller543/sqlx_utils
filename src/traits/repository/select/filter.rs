use std::fmt::Debug;
use crate::traits::{Model, Repository, SqlFilter};
use crate::types::Database;
use sqlx::{Database as DatabaseTrait, Executor, FromRow, QueryBuilder};
use std::future::Future;

pub trait FilterRepository<M>: Repository<M>
where
    M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
    Self: Sync,
{
    fn filter_query_builder<'args>() -> QueryBuilder<'args, Database>;

    #[inline(always)]
    #[tracing::instrument(skip(self, tx), level = "debug", parent = Self::repository_span(), name = "get_by_filter")]
    fn get_by_any_filter_with_executor<'a, F, E>(
        &'a self,
        tx: E,
        filter: F,
    ) -> impl Future<Output = crate::Result<Vec<M>>> + Send + 'a
    where
        F: for<'c> SqlFilter<'c, Database> + Debug + Send + 'a,
        E: for<'c> Executor<'c, Database = Database> + 'a,
    {
        async move {
            let mut query = Self::filter_query_builder();
            filter.apply_filter(&mut query);
            query.build_query_as().fetch_all(tx).await.map_err(Into::into)
        }
    }

    #[inline(always)]
    async fn get_by_any_filter<'a, F>(
        &'a self,
        filter: F,
    ) -> crate::Result<Vec<M>>
    where
        F: for<'c> SqlFilter<'c, Database> + Debug + Send + 'a,
    {
            let pool = self.pool();
            self.get_by_any_filter_with_executor(pool, filter).await
    }
}

pub trait FilterRepositoryExt<M, Filter>: FilterRepository<M>
where
    M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
    Filter: for<'args> SqlFilter<'args, Database> + Debug + Send,
    Self: Sync,
{
    #[inline(always)]
    async fn get_by_filter_with_executor<E>(
        &self,
        tx: E,
        filter: Filter,
    ) -> crate::Result<Vec<M>>
    where
        E: for<'c> Executor<'c, Database = Database>,
    {
        self.get_by_any_filter_with_executor(tx, filter).await
    }

    #[inline(always)]
    async fn get_by_filter(
        &self,
        filter: Filter,
    ) -> crate::Result<Vec<M>>
    {
        let pool = self.pool();
        self.get_by_any_filter_with_executor(pool, filter).await
    }
}

impl<M, Filter, T> FilterRepositoryExt<M, Filter> for T
where
    T: FilterRepository<M>,
    M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
    Filter: for<'args> SqlFilter<'args, Database> + Debug + Send,
    Self: Sync,
{}
