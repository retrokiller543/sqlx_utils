use sqlx_utils::sql_filter;

sql_filter! {
    pub struct UserFilter {
        FROM users WHERE  // Missing SELECT clause
        id = i32
    }
}

fn main() {
    let _filter = UserFilter::new(1);
}
