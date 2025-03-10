//! Model trait to define model specific methods

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
pub use sqlx_utils_macro::Model;

/// Trait for defining unique identification methods for database models.
///
/// The `Model` trait provides a standardized way to handle identification of database
/// models, with built-in support for collections and wrapper types like [`Vec`], [`Option`],
/// and [`Result`].
///
/// # Type Parameters
///
/// * [`Id`](Model::Id): The associated type representing the identifier type for the model.
///
/// # Examples
///
/// Basic implementation for a user model:
/// ```rust
/// # use sqlx_utils::traits::Model;
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// impl Model for User {
///     type Id = i32;
///
///     fn get_id(&self) -> Option<Self::Id> {
///         Some(self.id)
///     }
/// }
/// ```
///
/// Working with collections of models:
/// ```rust
/// # use sqlx_utils::traits::Model;
/// # struct User {
/// #     id: i32,
/// #     name: String,
/// # }
///
/// # impl Model for User {
/// #    type Id = i32;
/// #
/// #    fn get_id(&self) -> Option<Self::Id> {
/// #        Some(self.id)
/// #    }
/// # }
///
/// let users = vec![
///     User { id: 1, name: String::from("Alice") },
///     User { id: 2, name: String::from("Bob") }
/// ];
///
/// // Get IDs for all users in the vector
/// let ids = users.get_id(); // Returns Some(vec![Some(1), Some(2)])
/// ```
///
/// Handling optional models:
/// ```rust
/// # use sqlx_utils::traits::Model;
/// # struct User {
/// #     id: i32,
/// #     name: String,
/// # }
///
/// # impl Model for User {
/// #    type Id = i32;
/// #
/// #    fn get_id(&self) -> Option<Self::Id> {
/// #        Some(self.id)
/// #    }
/// # }
/// let maybe_user: Option<User> = Some(User { id: 1, name: String::from("Alice") });
/// let id = maybe_user.get_id(); // Returns Some(1)
///
/// let no_user: Option<User> = None;
/// let id = no_user.get_id(); // Returns None
/// ```
///
/// # Implementation Notes
///
/// The trait provides automatic implementations for:
///
/// * [`Vec<M>`]: Returns a vector of optional IDs
/// * [`Option<M>`]: Propagates the inner model's ID
/// * [`Result<M, E>`](Result): Returns the ID from Ok variants, None for Err
///
/// All implementations use `#[inline]` for optimal performance in tight loops
/// or when working with large collections of models.
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `Model` trait which is required for database operations",
    label = "this type does not implement `Model`",
    message = "`{Self}` must implement `Model` to define how to identify database records"
)]
pub trait Model: Send + Sync {
    /// The type used for model identification
    type Id: Send;

    /// Returns the model's identifier if available
    ///
    /// # Returns
    ///
    /// * [`Some(Id)`](Some) - If the model has an identifier
    /// * [`None`] - If the model has no identifier
    fn get_id(&self) -> Option<Self::Id>;

    fn has_id(&self) -> bool {
        self.get_id().is_some()
    }
}

impl<M> Model for Vec<M>
where
    M: Model + Send + Sync,
{
    type Id = Vec<Option<M::Id>>;

    #[inline]
    fn get_id(&self) -> Option<Self::Id> {
        let mut ids = Vec::new();

        for m in self {
            ids.push(m.get_id())
        }

        Some(ids)
    }
}

impl<M> Model for Option<M>
where
    M: Model + Send + Sync,
{
    type Id = M::Id;

    #[inline]
    fn get_id(&self) -> Option<Self::Id> {
        if let Some(m) = self {
            m.get_id()
        } else {
            None
        }
    }
}

impl<M, E> Model for Result<M, E>
where
    M: Model + Send + Sync,
    E: Send + Sync,
{
    type Id = M::Id;

    #[inline]
    fn get_id(&self) -> Option<Self::Id> {
        if let Ok(m) = self {
            m.get_id()
        } else {
            None
        }
    }
}

impl<K, V> Model for HashMap<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Model + Send + Sync,
{
    type Id = HashMap<K, Option<V::Id>>;

    #[inline]
    fn get_id(&self) -> Option<Self::Id> {
        Some(self.iter().map(|(k, v)| (k.clone(), v.get_id())).collect())
    }
}

impl<K, V> Model for BTreeMap<K, V>
where
    K: Ord + Clone + Send + Sync,
    V: Model + Send + Sync,
{
    type Id = BTreeMap<K, Option<V::Id>>;

    #[inline]
    fn get_id(&self) -> Option<Self::Id> {
        Some(self.iter().map(|(k, v)| (k.clone(), v.get_id())).collect())
    }
}
