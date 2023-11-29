#[get("/")]
pub async fn index(cookies: &CookieJar<'_>) -> Template {

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

#[get("/ys/uid/<uid>")]
async fn ys(uid: &str) -> String {

    let client = reqwest::Client::new();
    let response = client.get(format!("https://enka.network/api/uid/{}", uid))
        .send().await.unwrap();
    response.text().await.unwrap()
}

#[get("/buy?<num>")]
async fn buy(cookies: &CookieJar<'_>, num: i32) -> Template {

    let username = cookies.get_private("username").unwrap().to_string();
    let username = username[9..].to_string();

    let mut conn = POOL.get_conn().unwrap();
    let uid: Vec<i32> = conn.query(format!(r#"SELECT uid FROM users WHERE username = "{}""#, username)).unwrap();
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
        let if_guest = cookies.get_private("if_guest").unwrap().to_string();
        if if_guest.len() > 8 {
            if_guest[9..].to_string()
        } else {
            "1".to_string()
        }
    };
    
    if if_guest == "1" {
        return r#"{"message": "请登录", "retcode": "0"}"#;
    } else {
        let data = json::parse(data).unwrap();
        /*
        {
            "uid": uid,
            "num": num,
            "time": time,
            "amount": amount
        }
         */

        let mut conn = POOL.get_conn().unwrap();
        let mut flight_info: Vec<(i32, i32)> = conn.query(format!("
        SELECT capacity, booked
        FROM flights WHERE num = {}
        ", data["num"])).unwrap();

        println!("{:?}", flight_info);
        if flight_info.len() == 1 {
            let amount: i32 = match data["amount"].to_string().parse::<i32>() {
                Ok(parsed_amount) => parsed_amount,
                Err(e) => {
                    println!("{}", e);
                    println!("{:?}", data);
                    return r#"{"message": "传入数据错误", "retcode": "-1"}"#;
                }
            };
            if flight_info[0].0 - flight_info[0].1 >= amount {

                flight_info[0].1 += amount;
                conn.query_drop(format!("
                UPDATE flights SET booked = {} WHERE num = {}
                ", flight_info[0].1, data["num"])).unwrap();

                conn.query_drop(format!("
                INSERT INTO payments (num, uid, amount, time)
                VALUES ({}, {}, {}, {})
                ", data["num"], data["uid"], data["amount"], data["time"])).unwrap();

                return r#"{"message": "购买成功", "retcode": "1"}"#;
            } else {
                return r#"{"message": "购买数量大于剩余数量或者机票已经售完", "retcode": "2"}"#;
            }
        }
    }
    return r#"{"message": "未知错误", "retcode": "-1"}"#;
}

#[get("/search?<leave>&<arrive>")]
async fn search(leave: &str, arrive: &str) -> Template {

    let mut conn = POOL.get_conn().unwrap();

    let res: Vec<(i32, String, String, String, String, i64, i64, f64, i32, i32)> = conn.query("
        SELECT num, leave_city, arrive_city, leave_airport, arrive_airport, leave_time, arrive_time, price, capacity, booked
        FROM flights").unwrap();

    let mut result: Vec<(i32, String, String, String, String, i64, i64, f64, i32, i32)> = vec![];

    for i in &res {
        if i.to_owned().1 == leave && i.to_owned().2 == arrive {
            result.push((i.0, i.to_owned().1, i.to_owned().2, i.to_owned().3, i.to_owned().4, i.5, i.6, i.7, i.8, i.9));
        }
    }

    Template::render("search", context!{
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

    println!("{:?}", data);
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
            name: 123,
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

            println!("{:?}", data);

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

    
    Template::render("user", context!{
        name: 123,
    })
}