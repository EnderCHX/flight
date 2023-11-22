#[macro_use] extern crate rocket;
use json::{object, JsonValue};
use lazy_static::*;
use mysql::prelude::*;
use mysql::*;
use rocket::data;
use rocket::http::CookieJar;
use rocket::response::content::RawHtml;
use rocket::fs::{FileServer, relative};
use rocket::response::{status, Redirect};
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
async fn index(cookies: &CookieJar<'_>) -> Template {

    if cookies.get_private("username").is_none() {

        let timestamp: i64 = Utc::now().timestamp();
        let mut rng = rand::thread_rng();
        let randid = timestamp.to_string() + &rng.gen_range(0..10).to_string();

        cookies.add_private(("username", format!("guest{}", randid)));
        cookies.add_private(("if_guest", "1"));

    } else {
        return Template::render("index", context! {
            name: {
                let username = cookies.get_private("username").unwrap().to_string();
                if username.len() > 8 {
                    username[9..].to_string()
                } else {
                    "null".to_string()
                }
            },
            if_guest: {
                let if_guest = cookies.get_private("if_guest").unwrap().to_string();
                if if_guest.len() > 8 {
                    if_guest[9..].to_string()
                } else {
                    "null".to_string()
                }
            },
        });
    }
    Template::render("error", context! {
        error: "已退出登录，请刷新"
    })
}

#[get("/buy")]
async fn buy() -> RawHtml<&'static str> {
    RawHtml(r#"See <a href="tera">Tera</a> or <a href="hbs">Handlebars</a>.
                <img src="https://chxc.cc/img/chxw.png">
                "#)
}

#[get("/search?<leave>&<arrive>")]
async fn search(leave: &str, arrive: &str) -> Template {
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
async fn register() -> Template {
    Template::render("register", context!{
        name: 123,
    })
}

#[post("/register", format = "json", data = "<data>")]
async fn register_do(cookies: &CookieJar<'_>, data: String) -> &'static str {
    use json;
    let data = json::parse(&data).unwrap();
    let mut user = User {
        uid: -1,
        username: data["username"].to_string(),
        password: data["password"].to_string(),
        admin: false,
    };
    let mut conn= POOL.get_conn().unwrap();

    let status = user.user_register(&mut conn);


    if status == 1 {
        cookies.remove_private("username");
        cookies.add_private(("username", data["username"].to_string()));
        cookies.add_private(("if_guest", "0"));
        return "{\"message\": 1}"; //注册成功
    } else if status == 2 {
        return "{\"message\": 2}"; // 用户存在
    } else {
        return "{\"message\": 0}"; // 未知错误
    }
}

#[get("/login")]
async fn login() -> Template {
    Template::render("login", context!{
        name: 123
    })
}

#[post("/login", format = "json" ,data = "<data>")]
async fn login_do(cookies: &CookieJar<'_>, data: String) -> &'static str {
    use json;
    let data = json::parse(&data).unwrap();

    let mut conn = POOL.get_conn().unwrap();

    let mut user = User {
        uid: -1,
        username: data["username"].to_string(),
        password: data["password"].to_string(),
        admin: false,
    };

    let status_code = user.user_login(&mut conn);

    println!("{:?}", data);
    if status_code == 1 {
        cookies.remove_private("username");
        cookies.add_private(("username", data["username"].to_string()));
        cookies.add_private(("if_guest", "0"));
        return "{\"message\": 1}"; // 成功
    } else if status_code == 2 {
        return "{\"message\": 2}"; // 密码错误
    } else if status_code == -1 {
        return "{\"message\": -1}"; // 用户不存在
    } else {
        return "{\"message\": 0}"; // 未知错误
    }
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private("username");
    cookies.remove_private("if_guest");
    Redirect::to(uri!(index))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, buy, search, register, register_do, login, login_do, logout])
        .mount("/", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}
