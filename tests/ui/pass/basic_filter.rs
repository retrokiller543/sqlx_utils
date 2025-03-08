use sqlx_utils::sql_filter;

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM users WHERE
        id = i32
    }
}

fn main() {
    let _filter = UserFilter::new(1);
}
