#![allow(dead_code)]

use parking_lot::ArcMutexGuard;
use sqlx::{QueryBuilder, Transaction};
use sqlx_utils::prelude::*;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

pub static DATABASE_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("DATABASE_URL").expect("failed to get env var"));

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM users WHERE ?id = i64
    }
}

#[derive(Clone)]
pub struct User {
    id: i64,
    name: String,
}

impl Model for User {
    type Id = i64;

    fn get_id(&self) -> Option<i64> {
        Some(self.id)
    }
}

repository! {
    pub UserRepo<User>;
}

repository! {
    !zst
    pub UserRepo2<User>;
}

repository_insert! {
    UserRepo<User>;

    insert_query(model) {
        sqlx::query("INSERT INTO users (name) VALUES (?)").bind(&model.name)
    }
}

repository_update! {
    UserRepo<User>;

    update_query(model) {
        sqlx::query("UPDATE users SET name = ? where id = ?").bind(model.id).bind(&model.name)
    }
}

repository_delete! {
    UserRepo<User>;

    delete_by_id_query(id) {
        sqlx::query("DELETE FROM users WHERE id = ?").bind(id)
    }

    delete_by_filter_query(filter) {
        let mut builder = QueryBuilder::new("DELETE FROM users WHERE ");

        filter.apply_filter(&mut builder);

        builder
    }
}

pub enum UserContext {
    System,
    UnAuthenticated,
}

#[derive(Debug)]
pub enum DbError {
    SqlxError(sqlx::Error),
    SqlxUtils(sqlx_utils::Error),
    NotAllowed,
}

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self {
        DbError::SqlxError(e)
    }
}

impl From<sqlx_utils::Error> for DbError {
    fn from(e: sqlx_utils::Error) -> Self {
        DbError::SqlxUtils(e)
    }
}

impl UserRepo {
    pub async fn save_with_context(
        &self,
        model: User,
        context: UserContext,
    ) -> Result<User, DbError> {
        self.with_transaction(move |mut tx| async move {
            let res = match context {
                UserContext::System => self
                    .save_with_executor(&mut *tx, model)
                    .await
                    .map_err(Into::into),
                UserContext::UnAuthenticated => Err(DbError::NotAllowed),
            };

            (res, tx)
        })
        .await
    }

    pub async fn save_with_tx<'a, 'b>(&'a self, model: User) -> Result<Vec<User>, DbError> {
        self.transaction_sequential::<'a, 'b>([
            move |mut tx: Transaction<'b, Database>| async move {
                let res = self.save_with_executor(&mut *tx, model).await;

                (res, tx)
            },
        ])
        .await
        .map_err(Into::into)
    }

    pub async fn save_with_rx_concurrent<'a, 'b>(
        &'a self,
        model: User,
    ) -> Result<Vec<User>, DbError>
    where
        'b: 'a,
    {
        self.transaction_concurrent::<'a, 'b>([
            |tx: Arc<parking_lot::Mutex<Transaction<'b, Database>>>| async move {
                let mut tx = match tx.try_lock_arc() {
                    Some(tx) => tx,
                    None => return Err(Error::MutexLockError),
                };

                let res = USER_REPO.save_with_executor(&mut **tx, model).await;

                ArcMutexGuard::<parking_lot::RawMutex, Transaction<'b, sqlx::Any>>::unlock_fair(tx);

                res
            },
        ])
        .await
        .map_err(Into::into)
    }

    pub async fn try_save_with_tx<'a, 'b>(
        &'a self,
        model: User,
    ) -> Result<Vec<User>, Vec<DbError>> {
        self.try_transaction::<'a, 'b>([move |mut tx: Transaction<'b, Database>| async move {
            let res = self.save_with_executor(&mut *tx, model).await;

            (res, tx)
        }])
        .await
        .map_err(|errors| errors.into_iter().map(Into::into).collect())
    }
}

async fn action<'b>(
    _: Transaction<'b, Database>,
) -> (Result<User, DbError>, Transaction<'b, Database>) {
    unimplemented!()
}

#[tokio::main]
async fn main() {
    install_default_drivers();

    initialize_db_pool(
        PoolOptions::new()
            .max_connections(21)
            .min_connections(5)
            .idle_timeout(Duration::from_secs(60 * 10))
            .max_lifetime(Duration::from_secs(60 * 60 * 24))
            .acquire_timeout(Duration::from_secs(20))
            .connect(&DATABASE_URL)
            .await
            .expect("Failed to connect to database"),
    );

    let user = User {
        id: 1,
        name: String::new(),
    };

    USER_REPO.save_ref(&user).await.unwrap();

    USER_REPO.save_in_transaction(user.clone()).await.unwrap();

    USER_REPO
        .save_with_context(user.clone(), UserContext::System)
        .await
        .unwrap();

    USER_REPO.with_transaction(action).await.unwrap();

    USER_REPO
        .delete_by_values_in_transaction("id", [1, 2, 3, 11, 22])
        .await
        .unwrap();
}
