use sqlx_utils::sql_filter;

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM users WHERE
        age NOT NULL AND i32  // Invalid operator
    }
}

fn main() {
}
