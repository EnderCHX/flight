use mysql::*;
use mysql::prelude::*;

struct Payment {
    uid: u32,
    amount: i32,
    sum: f64,
}

struct Flight {
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

struct User {
    uid: u32,
    nickname: Option<String>,
    admin: bool,
}

fn main() {
    let url = "mysql://root:chxMIMA@@localhost:3306/test";
    let pool = Pool::new(url).unwrap(); // 获取连接池
    let mut conn = pool.get_conn().unwrap();// 获取链接
}
