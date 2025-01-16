use futures::future::try_join_all;
use futures::FutureExt;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use tracing::instrument;

pub(crate) const DEFAULT_BATCH_SIZE: usize = 256;

#[derive(Debug)]
pub(crate) struct BatchOperator<T, const N: usize = DEFAULT_BATCH_SIZE>(Vec<T>);

impl<T, const N: usize> Deref for BatchOperator<T, N> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for BatchOperator<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const N: usize> Default for BatchOperator<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> BatchOperator<T, N> {
    pub fn new() -> Self {
        Self(Vec::with_capacity(N))
    }

    async fn execute_query_internal<'a>(
        items: &'a mut Vec<T>,
        pool: & crate::types::Pool,
        query: fn(&T) -> crate::types::Query,
    ) -> crate::Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let mut tx = pool.begin().await?;

        for item in items.drain(..) {
            query(&item).execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn execute_query(
        iter: impl IntoIterator<Item = T>,
        pool: &crate::types::Pool,
        query: fn(&T) -> crate::types::Query,
    ) -> crate::Result<()> {
        let mut buf = Self::new();

        for item in iter {
            buf.push(item);

            if buf.len() == N {
                Self::execute_query_internal(&mut buf.0, pool, query).await?;
            }
        }

        Self::execute_query_internal(&mut buf.0, pool, query).await?;

        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn execute_batch<F, Fut, E>(
        iter: impl IntoIterator<Item = T>,
        worker: F,
    ) -> Result<(), E>
    where
        F: Fn(Vec<T>) -> Fut,
        Fut: Future<Output = Result<(), E>>,
    {
        let mut buf = Self::new();
        let mut futures = Vec::new();

        for item in iter {
            buf.push(item);
            if buf.len() == N {
                futures.push(worker(buf.drain(..).collect()));
            }
        }

        if !buf.is_empty() {
            futures.push(worker(buf.drain(..).collect()));
        }

        try_join_all(futures).await?;
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    /// # NOTE: This only works in a non Send context
    pub async fn partition_execute<F1, F2, Fut1, Fut2, P, E>(
        iter: impl IntoIterator<Item = T>,
        predicate: P,
        worker1: F1,
        worker2: F2,
    ) -> Result<(), E>
    where
        P: Fn(&T) -> bool,
        F1: Fn(Vec<T>) -> Fut1,
        F2: Fn(Vec<T>) -> Fut2,
        Fut1: Future<Output = Result<(), E>>,
        Fut2: Future<Output = Result<(), E>>,
    {
        let mut buf1 = Self::new();
        let mut buf2 = Self::new();
        let mut futures = Vec::new();

        for item in iter {
            if predicate(&item) {
                buf1.push(item);
                if buf1.len() == N {
                    futures.push(worker1(buf1.drain(..).collect()).boxed_local());
                }
            } else {
                buf2.push(item);
                if buf2.len() == N {
                    futures.push(worker2(buf2.drain(..).collect()).boxed_local());
                }
            }
        }

        if !buf1.is_empty() {
            futures.push(worker1(buf1.drain(..).collect()).boxed_local());
        }
        if !buf2.is_empty() {
            futures.push(worker2(buf2.drain(..).collect()).boxed_local());
        }

        try_join_all(futures).await?;

        Ok(())
    }
}
