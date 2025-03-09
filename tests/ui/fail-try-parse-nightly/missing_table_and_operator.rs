use sqlx_utils::sql_filter;

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM WHERE  // Missing FROM clause
        (id EQUALS i32)
    }
}

fn main() {
}
