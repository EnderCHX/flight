use crate::initdb::DbInfo;
pub struct Flight {
    num: u32,
    capacity: u32,
    booked: u32,
    price: f64,
    leave_city: Option<String>,
    arrive_city: Option<String>,
    leave_airport: Option<String>,
    arrive_airport: Option<String>,
    leave_time: String,
    arrive_time: String,
}

impl Flight {
    pub fn search() {

    }
}