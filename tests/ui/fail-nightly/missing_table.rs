use sqlx_utils::sql_filter;

sql_filter! {
    pub struct UserFilter {
        SELECT * WHERE  // Missing FROM clause
        id = i32
    }
}

fn main() {
}
