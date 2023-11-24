use crate::initdb::DbInfo;
pub struct Flight {
    num: i32,
    capacity: i32,
    booked: i32,
    price: f64,
    leave_city: Option<String>,
    arrive_city: Option<String>,
    leave_airport: Option<String>,
    arrive_airport: Option<String>,
    leave_time: i64,
    arrive_time: i64,
}

impl Flight {
    pub fn search() {

    }
}