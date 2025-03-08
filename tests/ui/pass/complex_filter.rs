use sqlx_utils::sql_filter;

sql_filter! {
    pub struct OrderFilter {
        SELECT id, total, name as customer_name FROM orders WHERE
        (id = i32 OR customer_id = i32) AND
        ?total > f64 AND
        ?created_at > "NOW() - INTERVAL '1 day'"
    }
}

fn main() {
    let _filter = OrderFilter::new(1, 2).total(100.0);
}
