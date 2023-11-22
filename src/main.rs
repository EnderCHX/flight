#[macro_use] extern crate rocket;
use std::ptr::null;
use lazy_static::*;
use mysql::prelude::Queryable;
use mysql::{PooledConn, Pool, params};
use rocket::http::CookieJar;
use rocket::http::hyper::server::conn;
use rocket::response::content::RawHtml;
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::*;
use rand::Rng;
use initdb::DbInfo;
use chrono::*;
use user::User;

mod initdb;
mod user;
mod flight;
mod payment;

lazy_static! {
    static ref DB: DbInfo = DbInfo {
        ip: String::from("localhost"),
        port: String::from("3306"),
        user: String::from("root"),
        password: String::from("chxMIMA@"),
        db_name: String::from("db_flight"),
    };
    static ref POOL: Pool = {
        DB.init_db();
        let url = format!("mysql://{}:{}@{}:{}/{}", DB.user, DB.password, DB.ip, DB.port, DB.db_name);
        let pool = Pool::new(url.as_str()).unwrap();
        pool
    };
}



#[get("/")]
fn index(cookies: &CookieJar<'_>) -> Template {
    if cookies.get("username").is_none() {
        let timestamp: i64 = Utc::now().timestamp();
        let mut rng = rand::thread_rng();
        let randid = timestamp.to_string() + &rng.gen_range(0..10).to_string();
        cookies.add_private(("username", format!("guest{}", randid)));
    }
    Template::render("index", context! {
        name: 123,
    })
}

#[get("/buy")]
fn buy() -> RawHtml<&'static str> {
    RawHtml(r#"See <a href="tera">Tera</a> or <a href="hbs">Handlebars</a>.
                <img src="https://chxc.cc/img/chxw.png">
                "#)
}

#[get("/search?<leave>&<arrive>")]
fn search(leave: &str, arrive: &str) -> Template {
    println!("{} {}", leave, arrive);
    let mut conn = POOL.get_conn().unwrap();
    let res: Vec<(u32, String, String, String, String, u32, u32, u32, u32, u32)> = conn.query("
        SELECT num, leave_city, arrive_city, leave_airport, arrive_airport, leave_time, arrive_time, price, capacity, booked
        FROM flights").unwrap();
    let mut result: Vec<(u32, String, String, String, String, u32, u32, u32, u32, u32)> = vec![];
    for i in res {
        if i.1 == leave && i.2 == arrive {
            result.push((i.0, i.1, i.2, i.3, i.4, i.5, i.6, i.7, i.8, i.9));
        }
    }
    Template::render("search", context!{
        result: &result,
        ifres: result.len(),
    })
}

#[get("/register")]
fn register() -> Template {
    Template::render("register", context!{
        name: 123,
    })
}

#[get("/register?<username>&<password>")]
fn register_do(username: String, password: String,cookies: &CookieJar<'_>) -> &'static str {
    let mut user = User {
        uid: 0,
        username: username,
        password: password,
        admin: false,
    };
    let mut conn= POOL.get_conn().unwrap();
    let status = user.user_register(&mut conn);
    cookies.remove("username");
    cookies.add_private(("username", user.username));
    if status == true {
        return "注册成功";
    } else {
        return "注册失败";
    }
}

#[get("/login")]
fn login() {
    
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, buy, search, register, register_do])
        .mount("/", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}
