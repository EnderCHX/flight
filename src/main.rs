use chrono::format::format;
use initdb::DbInfo;
use mysql::*;
use mysql::prelude::*;
use rocket::response::content::RawHtml;

use crate::user::User;

mod initdb;
mod user;
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





// #[macro_use] extern crate rocket;

// #[get("/")]
// fn index() -> &'static str {
//     "Hello, world!"
// }

// #[get("/buy")]
// fn buy() -> RawHtml<&'static str> {
//     RawHtml(r#"See <a href="tera">Tera</a> or <a href="hbs">Handlebars</a>.
//                 <img src="https://chxc.cc/img/chxw.png">
//                 "#)
// }

// #[launch]
// fn rocket() -> _ {
//     rocket::build().mount("/", routes![index, buy])
// }

fn main() {
    let db = DbInfo {
        ip: "localhost".to_owned(),
        port: "3306".to_owned(),
        user: "root".to_owned(),
        password: "chxMIMA@".to_owned(),
        db_name: "db_flight".to_owned(),
    };
    db.init_db();
    let mut conn = db.conn();
}