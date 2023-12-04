#[macro_use] extern crate rocket;
use json::JsonValue;
use lazy_static::*;
use mysql::prelude::*;
use mysql::*;
use payment::Payment;
use rocket::http::CookieJar;
use rocket::fs::{FileServer, relative};
use rocket::response::Redirect;
use rocket_dyn_templates::*;
use rand::Rng;
use initdb::DbInfo;
use chrono::*;
use user::User;
use flight::Flight;


mod cors;
mod initdb;
mod user;
mod flight;
mod payment;


lazy_static! {

    static ref CONFIG: JsonValue = {

        let config = std::fs::read_to_string("config.json").expect("读取配置失败");
        json::parse(config.as_str()).expect("json转换失败")

    };

    static ref DB: DbInfo =  DbInfo {
        ip: CONFIG["mysql"]["ip"].to_string(),
        port: CONFIG["mysql"]["port"].to_string(),
        user: CONFIG["mysql"]["user"].to_string(),
        password: CONFIG["mysql"]["password"].to_string(),
        db_name: CONFIG["mysql"]["db_name"].to_string(),
    };

    static ref POOL: Pool = {
        DB.init_db();
        let url = format!("mysql://{}:{}@{}:{}/{}", DB.user, DB.password, DB.ip, DB.port, DB.db_name);
        let pool = Pool::new(url.as_str()).expect("创建连接池失败");
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

        Redirect::to(uri!(index));

    } else {
        return Template::render("index", context! {
            name: {
                let username = cookies.get_private("username").expect("获取cookie失败").to_string();
                if username.len() > 8 {
                    username[9..].to_string()
                } else {
                    "null".to_string()
                }
            },
            if_guest: {
                let if_guest = cookies.get_private("if_guest").expect("获取cookie失败").to_string();
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

#[get("/ys/uid/<uid>")]
async fn ys(uid: &str) -> String {

    let client = reqwest::Client::new();
    let response = client.get(format!("https://enka.network/api/uid/{}", uid))
        .send().await.expect("ys请求失败");
    response.text().await.expect("响应失败")
}

#[get("/buy?<num>")]
async fn buy(cookies: &CookieJar<'_>, num: i32) -> Template {

    let username = cookies.get_private("username").expect("获取cookie失败").to_string();
    let username = username[9..].to_string();

    let mut conn = POOL.get_conn().expect("数据库连接失败");
    let uid: Vec<i32> = conn.query(format!(r#"SELECT uid FROM users WHERE username = "{}""#, username)).expect("数据库查询失败");
    let uid = {if uid.len() == 1 {uid[0]} else {0}};

    let time = Utc::now().timestamp();

    Template::render("buy", context!{
        uid: uid,
        num: num,
        time: time,
    })
}

#[post("/pay", format = "json", data = "<data>")]
async fn pay(cookies: &CookieJar<'_>, data: &str) -> &'static str {

    let if_guest = {
        let if_guest = cookies.get_private("if_guest").expect("获取cookie失败").to_string();
        if if_guest.len() > 8 {
            if_guest[9..].to_string()
        } else {
            "1".to_string()
        }
    };
    
    if if_guest == "1" {
        return r#"{"message": "请登录", "retcode": "0"}"#;
    } else {
        let data = json::parse(data).expect("获取cookie失败");
        /*
        {
            "uid": uid,
            "num": num,
            "time": time,
            "amount": amount
        }
         */

        let payment = Payment {
            uid: match data["uid"].to_string().parse::<i32>() {
                Ok(uid) => uid,
                Err(e) => {
                    println!("{}", e);
                    -1
                }
            }, 
            num: match data["num"].to_string().parse::<i32>() {
                Ok(num) => num,
                Err(e) => {
                    println!("{}", e);
                    -1
                }
            },
            amount: match data["amount"].to_string().parse::<i32>() {
                Ok(amount) => amount,
                Err(e) => {
                    println!("{}", e);
                    -1
                }
            },
            time: match data["time"].to_string().parse::<i64>() {
                Ok(time) => time,
                Err(e) => {
                    println!("{}", e);
                    -1
                }
            }
        };

        let mut conn = POOL.get_conn().expect("数据库连接失败");
        let mut flight_info: Vec<(i32, i32)> = conn.query(format!("
        SELECT capacity, booked
        FROM flights WHERE num = {}
        ", payment.num)).expect("查询失败");

        if flight_info.len() == 1 {
            
            if flight_info[0].0 - flight_info[0].1 >= payment.amount {

                flight_info[0].1 += payment.amount;
                conn.query_drop(format!("
                UPDATE flights SET booked = {} WHERE num = {}
                ", flight_info[0].1, data["num"])).expect("数据库更新失败");

                if data["amount"].to_string() == "" {
                    return r#"{"message": "请输入数量", "retcode": "-1"}"#;
                }

                conn.query_drop(format!("
                INSERT INTO payments (num, uid, amount, time)
                VALUES ({}, {}, {}, {})
                ", data["num"], data["uid"], data["amount"], data["time"])).expect("数据库插入失败");

                return r#"{"message": "购买成功", "retcode": "1"}"#;
            } else {
                return r#"{"message": "购买数量大于剩余数量或者机票已经售完", "retcode": "2"}"#;
            }
        }
    }
    return r#"{"message": "未知错误", "retcode": "-1"}"#;
}

#[get("/search?<leave>&<arrive>")]
async fn search(cookies: &CookieJar<'_>,leave: &str, arrive: &str) -> Template {

    let mut conn = POOL.get_conn().unwrap();

    let res: Vec<(i32, String, String, String, String, i64, i64, f64, i32, i32)> = conn.query("
        SELECT num, leave_city, arrive_city, leave_airport, arrive_airport, leave_time, arrive_time, price, capacity, booked
        FROM flights").unwrap();

    let mut result: Vec<(i32, String, String, String, String, i64, i64, f64, i32, i32)> = vec![];

    for i in &res {
        if i.to_owned().1 == leave && i.to_owned().2 == arrive {
            
            let _f = Flight {
                num: i.0,
                capacity: i.8,
                booked: i.9,
                price: i.7,
                leave_city: Some(i.to_owned().1),
                arrive_city: Some(i.to_owned().2),
                leave_airport: Some(i.to_owned().3),
                arrive_airport: Some(i.to_owned().4),
                leave_time: i.5, 
                arrive_time: i.6, 
            };

            result.push((i.0, i.to_owned().1, i.to_owned().2, i.to_owned().3, i.to_owned().4, i.5, i.6, i.7, i.8, i.9));
        }
    }

    Template::render("search", context!{
        name: {
            let username = cookies.get_private("username").expect("获取cookie失败").to_string();
            if username.len() > 8 {
                username[9..].to_string()
            } else {
                "null".to_string()
            }
        },
        leave: leave,
        arrive: arrive,
        result: &result,
        ifres: result.len(),
        all: res,
        if_all: {if leave == "all" && arrive == "all" {true} else {false}},
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
        return r#"{"message": "注册成功", "retcode": 1}"#; //注册成功
    } else if status == 2 {
        return r#"{"message": "用户存在", "retcode": 2}"#; // 用户存在
    } else {
        return r#"{"message": "未知错误", "retcode": -1}"#; // 未知错误
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

    if status_code == 1 {
        cookies.remove_private("username");
        cookies.add_private(("username", data["username"].to_string()));
        cookies.add_private(("if_guest", "0"));
        return r#"{"message": "成功", "retcode": 1}"#; // 成功
    } else if status_code == 2 {
        return r#"{"message": "密码错误", "retcode": 2}"#; // 密码错误
    } else if status_code == -1 {
        return r#"{"message": "用户不存在", "retcode": 0}"#; // 用户不存在
    } else {
        return r#"{"message": "未知错误", "retcode": -1}"#; // 未知错误
    }
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private("username");
    cookies.remove_private("if_guest");
    Redirect::to(uri!(index))
}

#[get("/admin")]
async fn admin(cookies: &CookieJar<'_>) -> Template {

    let get_name = cookies.get_private("username").unwrap().to_string();

    let mut user = User {
        uid: -1,
        username: get_name[9..].to_string(),
        password: String::new(),
        admin: false
    };

    let mut conn = POOL.get_conn().unwrap();
    conn.query_map(format!("
        SELECT uid, admin FROM users WHERE username = '{}'", 
        user.username),
        | (uid, admin)| {
            user.uid = uid;
            user.admin = admin;
        }).unwrap();
    
    if user.admin {

        let flights_list: Vec<(i32, String, String, String, String, i64, i64, f64, i32, i32)> = conn.query("
            SELECT * FROM flights").unwrap();

        let mut users_list: Vec<(i32, String, String, bool)> = conn.query(r#"SELECT * FROM users"#).unwrap();

        for i in 0..users_list.len() {

            users_list[i].2 = "Do not see".to_string();

        }
        
        return Template::render("admin", context!{
            name: {
                let username = cookies.get_private("username").expect("获取cookie失败").to_string();
                if username.len() > 8 {
                    username[9..].to_string()
                } else {
                    "null".to_string()
                }
            },
            flight_num: flights_list.len(),
            flights: flights_list,
            users: users_list,
        })
    } else {
        return Template::render("error", context!{
            error: "你不是管理员",
        });
    }
}

#[post("/admin/cflight", format = "json", data = "<data>")]
fn change_flight(cookies: &CookieJar<'_>, data: &str) -> &'static str {
    use json;

    
    /*{
        "type": 0, // 0添加 1修改 2删除
        "info": {
            "num": 1,
            "leave_city": "哈尔滨",
            "arrive_city": "石家庄",
            "leave_airport": "太平国际机场",
            "arrive_airport": "正定机场",
            "leave_time": 时间戳,
            "arrive_time": 时间戳,
            "price": 100,
            "capacity": 100,
            "booked": 10
        }
    } */

    let mut conn = POOL.get_conn().unwrap();

    let username = cookies.get_private("username").unwrap().to_string();
    let username = username[9..].to_string();

    let if_admin: Vec<bool> = conn.query(format!("
        SELECT admin FROM users WHERE username = '{}'", username
        )).unwrap();
    
    if if_admin.len() == 1 {
        if if_admin[0] {
            let data = json::parse(data).unwrap(); 

            if data["type"] == 0 {

                conn.query_drop(format!(r#"
                INSERT INTO flights values({}, "{}", "{}", "{}", "{}", 
                {}, {}, {}, {}, {})"#,
                data["info"]["num"],        data["info"]["leave_city"],  data["info"]["arrive_city"], data["info"]["leave_airport"], data["info"]["arrive_airport"], 
                data["info"]["leave_time"], data["info"]["arrive_time"], data["info"]["price"],       data["info"]["capacity"],      data["info"]["booked"]
                )).unwrap();

                return r#"{"message": "新增成功", "retcode": "1"}"#; // 新增成功

            } else if data["type"] == 1 {

                conn.query_drop(format!(r#"
                UPDATE flights SET leave_city = "{}", arrive_city = "{}", leave_airport = "{}", arrive_airport = "{}", 
                leave_time = {}, arrive_time = {}, price = {}, capacity = {}, booked = {}
                WHERE num = {}"#,
                data["info"]["leave_city"],  data["info"]["arrive_city"], data["info"]["leave_airport"], data["info"]["arrive_airport"], 
                data["info"]["leave_time"], data["info"]["arrive_time"], data["info"]["price"], data["info"]["capacity"], data["info"]["booked"], data["info"]["num"]
                )).unwrap();

                return r#"{"message": "修改成功", "retcode": "2"}"#; // 修改成功

            } else if data["type"] == 2 {

                conn.query_drop(format!(r#"
                DELETE FROM flights 
                WHERE num = {}"#, data["info"]["num"]
                )).unwrap();

                return r#"{"message": "删除成功", "retcode": "3"}"#; //删除成功

            } else {

                return r#"{"message": "未知操作类型", "retcode": "-1"}"#; // 未知操作类型

            }
        } else {

            return r#"{"message": "不是管理员不能修改", "retcode": "0"}"#; //不是管理员不能修改

        }
    }

    return r#"{"message": "未知错误", "retcode": "-1"}"#; // 未知错误
}

#[post("/admin/cuser", format = "json", data = "<data>")]
async fn change_user(cookies: &CookieJar<'_>, data: &str) -> &'static str {
    use json;

    let mut conn = POOL.get_conn().unwrap();

    let username = cookies.get_private("username").unwrap().to_string();
    let username = username[9..].to_string();

    let if_admin: Vec<bool> = conn.query(format!("
        SELECT admin FROM users WHERE username = '{}'", username
    )).unwrap();
    
    if if_admin.len() == 1 {

        if if_admin[0] {
            
            let data = json::parse(data).unwrap();

            let cuser = User {
                uid: data["uid"].to_string().parse::<i32>().unwrap(),
                username: data["username"].to_string(),
                password: data["password"].to_string(),
                admin: data["admin"].to_string().parse::<bool>().unwrap(),
            };

            if cuser.password == "11f08230c4ebb3a9b839e4e5a3cbbb10" {

                conn.query_drop(format!(r#"
                UPDATE IGNORE users SET username = "{}", admin = "{}" WHERE uid = {}"#,
                cuser.username.as_str(), {if cuser.admin {1} else {0}}, cuser.uid,
                )).unwrap();

            } else {

                conn.query_drop(format!(r#"
                UPDATE IGNORE users SET username = "{}", password = "{}", admin = "{}" WHERE uid = {}"#,
                cuser.username.as_str(), cuser.password.as_str(), {if cuser.admin {1} else {0}}, cuser.uid,
                )).unwrap();

            }
            

            if conn.affected_rows() > 0 {

                return r#"{"message": "成功", "retcode": "1"}"#; // 成功

            } else {

                return r#"{"message": "成功，数据未更改", "retcode": "2"}"#; // 成功，数据未更改

            }

        } else {

            return r#"{"message": "不是管理员不能修改", "retcode": "0"}"#; //不是管理员不能修改
        }
    }
    
    return r#"{"message": "未知错误", "retcode": "-1"}"#; // 未知错误
}

#[get("/user")]
async fn user_(cookies: &CookieJar<'_>) -> Template {

    let get_name = cookies.get_private("username").unwrap().to_string();

    let mut user = User {
        uid: -1,
        username: get_name[9..].to_string(),
        password: String::new(),
        admin: false
    };

    let mut conn = POOL.get_conn().expect("数据库连接失败");

    let info: Vec<(i32, i32, i32, i64, i32, String, String, bool)> = conn.query(format!(r#"
        SELECT * FROM payments
        JOIN users ON users.uid = payments.uid AND username = "{}"
        "#, user.username)).expect("error in user()");
    
    if info.len() > 0 {
        user.uid = info[0].0;
        user.username = info[0].5.clone();
        user.password = String::from("Do not see");
        user.admin = info[0].7;
    } else {
        conn.query_map(format!("
            SELECT uid, admin FROM users WHERE username = '{}'", 
            user.username),
            | (uid, admin)| {
                user.uid = uid;
                user.admin = admin;
            }).unwrap();
    }

    let mut pay_list: Vec<(i32, i32, i64)> = Vec::new();
   

    for i in info {
        pay_list.push((i.1, i.2, i.3));
    }

    Template::render("user", context!{
        username: user.username,
        uid: user.uid,
        pay_list: pay_list,
    })
}

#[post("/user", format = "json", data = "<data>")]
async fn user_do(cookies: &CookieJar<'_>, data: &str) -> &'static str {

    let get_name = cookies.get_private("username").expect("获取cookie失败").to_string();

    let mut user = User {
        uid: -1,
        username: get_name[9..].to_string(),
        password: String::new(),
        admin: false
    };

    let mut conn = POOL.get_conn().unwrap();

    conn.query_map(format!("
        SELECT uid, password, admin FROM users WHERE username = '{}'", 
        user.username),
        | (uid, password, admin)| {
            user.uid = uid;
            user.password = password;
            user.admin = admin;
        }).expect("查询数据库失败");

    let data = json::parse(data).expect("json数据转换失败");
    
    /*
    {
        "type": "1\2\", //drop payments, change username && change passwd
        "data": {
            "username": new name,
            "old_password": hash,
            "new_password": hash,
            "re_new_password": hash,
        },
        "data": {
            "uid": uid,
            "num": num,
            "time": timestamp
        }
    }
     */

    if data["type"].to_string() == "1" {

        let uid_in = match data["data"]["uid"].to_string().parse::<i32>() {
            Ok(uid) => uid,
            Err(e) => {
                println!("{}", e);
                0
            }
        };

        if uid_in == user.uid {
            
            let mut amount = 0;

            conn.query_map(format!("
                SELECT amount FROM payments WHERE uid = {} AND num = {} AND time = {}", 
                data["data"]["uid"], data["data"]["num"], data["data"]["time"]), | e | {amount = e;} ).unwrap();

            let mut booked = 0;

            conn.query_map(format!("
                SELECT booked FROM flights WHERE num = {}", 
                data["data"]["num"]), | e | {booked = e;} ).expect("查询失败");
            
            let booked = booked - amount;

            conn.query_drop(format!("UPDATE flights SET booked = {} WHERE num = {}", booked, data["data"]["num"])).unwrap();

            conn.query_drop(format!("
                DELETE FROM payments
                WHERE uid = {} AND num = {} AND time = {}
                ", data["data"]["uid"], data["data"]["num"], data["data"]["time"])).expect("数据库删除失败");

            if conn.affected_rows() > 0 {

                return r#"{"message": "删除成功", "retcode": "1"}"#;

            } else {

                return r#"{"message": "删除失败，数据库操作失败", "retcode": "-1"}"#;

            }
            
        } else {

            return r#"{"message": "用户订单不匹配", "retcode": "-1"}"#;

        }

    } else if data["type"].to_string() == "2" {
        
        if data["data"]["old_password"].to_string() == user.password {

            if data["data"]["new_password"] == data["data"]["re_new_password"] {

                conn.query_drop(format!(r#"
                UPDATE users SET username = "{}", password = "{}" WHERE uid = {}
                "#, data["data"]["username"], data["data"]["new_password"], user.uid)).expect("数据库更新失败");

                if conn.affected_rows() > 0 {

                    cookies.remove_private("username");
                    cookies.add_private(("username", data["data"]["username"].to_string()));
                    
                    return r#"{"message": "修改成功", "retcode": "2"}"#;

                } else {
                    
                    return r#"{"message": "修改失败，数据库操作失败；或未做更改", "retcode": "-2"}"#;

                }

            } else {

                return r#"{"message": "密码不一致", "retcode": "-2"}"#;

            }
        } else {

            return r#"{"message": "密码错误", "retcode": "-1"}"#;

        }

    } else {
        return r#"{"message": "未知操作类型", "retcode": "0"}"#;
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, ys, 
                                  buy, pay,
                                  search, 
                                  register, 
                                  register_do, 
                                  login, 
                                  login_do, 
                                  logout, 
                                  admin, user_,
                                  user_do,
                                  change_flight,
                                  change_user])
        .mount("/", FileServer::from(relative!("static")))
        .attach(Template::fairing())
        .attach(cors::get_cors())
}
