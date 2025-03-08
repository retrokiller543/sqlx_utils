use sqlx_utils::sql_filter;

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM users WHERE
        ?name LIKE String AND
        ?age >= i32
    }
}

fn main() {
    let filter = UserFilter::new();
    let _with_name = filter.name("John%");

    let filter2 = UserFilter::new().age(18);
    let _with_both = filter2.name("Jane%");
}
