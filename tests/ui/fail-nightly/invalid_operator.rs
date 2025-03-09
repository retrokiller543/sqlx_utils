use sqlx_utils::sql_filter;

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM users WHERE
        age INVALID i32  // Invalid operator
    }
}

fn main() {
}
