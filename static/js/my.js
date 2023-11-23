//值是否为空
function ifnull(id) {
    if(document.getElementById(id).value==''){
        return "null";
    } else {
        return document.getElementById(id).value;
    }
}

// 简写id获取控件
getE = (id) =>{
    return document.getElementById(id);
}

//查询航班
function search_flight() {
    let arrive = ifnull("arrive");
    let leave = ifnull("leave");
    let domain = window.location.host;
    window.location.href = `http://${domain}/search?leave=${leave}&arrive=${arrive}`;
}

//返回首页
function home() {
    let home = window.location.host;
    window.location.href = `http://${home}`;
}

function sleep (time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}

//注册
function register() {
    console.log("hello");
    let username = document.getElementById("username").value;
    let password = document.getElementById("password").value;
    let password2 = document.getElementById("password2").value;
    if (username == "") {
        document.getElementById("info").innerHTML = "用户名不能为空";
    } else {
        if (password == password2) {
            let hash = md5(password); //备用"skOR7oRda5iypO1ejFnmyd2MkDOlYUHG0STBMDBDTXo="
            
            const data = {
                username: username,
                password: hash
            };

            fetch(`http://${window.location.host}/register`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(data),
            })
            .then((response) => response.json())
            .then((data) => {
                console.log("Success:", data);
                if (data["message"] == 1) {
                    document.getElementById("info").innerHTML = "注册成功，准备跳转";
                    sleep(2000).then(() => {
                        home();
                    })
                } else if (data["message"] == 2) {
                    document.getElementById("info").innerHTML = "注册失败，用户存在";
                } else {
                    document.getElementById("info").innerHTML = "登录失败，未知错误";
                }
            })
            .catch((error) => {
                console.error("Error:", error);
            });
        } else {
            document.getElementById("info").innerHTML = "密码不一致";
        }
    }
}

// 登录
function login() {
    console.log("hello");
    let username = document.getElementById("username").value;
    let password = document.getElementById("password").value;
    if (username == "") {
        document.getElementById("info").innerHTML = "用户名不能为空";
    } else {
        document.getElementById("info").innerHTML = "正在登录";
        sleep(2000).then(() => {
            let hash = md5(password); 
            const data = {
                username: username,
                password: hash
            };

            fetch(`http://${window.location.host}/login`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(data),
            })
            .then((response) => response.json())
            .then((data) => {
                console.log("Success:", data);
                if (data["message"] == 1) {
                    document.getElementById("info").innerHTML = "登录成功，准备跳转";
                    sleep(2000).then(() => {
                        home();
                    })
                } else if (data["message"] == 2) {
                    document.getElementById("info").innerHTML = "登录失败，用户名或者密码错误";
                } else if (data["message"] == -1) {
                    document.getElementById("info").innerHTML = "登录失败，用户不存在";
                } else {
                    document.getElementById("info").innerHTML = "登录失败，未知错误";
                }
            })
            .catch((error) => {
                console.error("Error:", error);
            });
        })
    }
}

function logout() {
    fetch(`http://${window.location.host}/logout`)
    .then((response => { 
        console.log(response);
        console.log("logout"); 
        home();
    }))
}

function change_flight(e) {
    let info = {
        "num": getE(`num_${e}`).value,
        "leave_city": getE(`leave_city_${e}`).value,
        "arrive_city": getE(`arrive_city_${e}`).value,
        "leave_airport": getE(`leave_airport_${e}`).value,
        "arrive_airport": getE(`arrive_airport_${e}`).value,
        "leave_time": getE(`leave_time_${e}`).value,
        "arrive_time": getE(`arrive_time_${e}`).value,
        "price": getE(`price_${e}`).value,
        "capacity": getE(`capacity_${e}`).value,
        "booked": getE(`booked_${e}`).value,
    };
    console.log(info)
    
    let data = {
        "type": 0,
        "info": info
    }
    console.log(data)

    fetch(`http://${window.location.host}/admin/cflight`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
    }).then((response) => response.json())
    .then((data) => {
        console.log(data);
    })
    .catch((error) => {
        console.error("Error:", error);
    });
}