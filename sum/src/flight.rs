pub struct Flight {
    pub num: i32,
    pub capacity: i32,
    pub booked: i32,
    pub price: f64,
    pub leave_city: Option<String>,
    pub arrive_city: Option<String>,
    pub leave_airport: Option<String>,
    pub arrive_airport: Option<String>,
    pub leave_time: i64,
    pub arrive_time: i64,
}
