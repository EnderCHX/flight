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
function search_flight(all) {
    if (all == "all") {
        window.location.href = `/search?leave=all&arrive=all`;
    } else {
        let arrive = ifnull("arrive");
        let leave = ifnull("leave");
        window.location.href = `/search?leave=${leave}&arrive=${arrive}`;
    }
}

//返回首页
function home() {
    window.location.href = `/`;
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

            fetch(`/register`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(data),
            })
            .then((response) => response.json())
            .then((data) => {
                console.log("Success:", data);
                if (data["retcode"] == 1) {
                    document.getElementById("info").innerHTML = "注册成功，准备跳转";
                    sleep(2000).then(() => {
                        home();
                    })
                } else if (data["retcode"] == 2) {
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

            fetch(`/login`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(data),
            })
            .then((response) => response.json())
            .then((data) => {
                console.log("Success:", data);
                if (data["retcode"] == 1) {
                    document.getElementById("info").innerHTML = "登录成功，准备跳转";
                    sleep(2000).then(() => {
                        home();
                    })
                } else if (data["retcode"] == 2) {
                    document.getElementById("info").innerHTML = "登录失败，用户名或者密码错误";
                } else if (data["retcode"] == 0) {
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
    fetch(`/logout`)
    .then((response => { 
        console.log(response);
        console.log("logout"); 
        home();
    }))
}

function change_flight(e, type) {
    let info = {
        "num":              getE(`num_${e}`).value,
        "leave_city":       getE(`leave_city_${e}`).value,
        "arrive_city":      getE(`arrive_city_${e}`).value,
        "leave_airport":    getE(`leave_airport_${e}`).value,
        "arrive_airport":   getE(`arrive_airport_${e}`).value,
        "leave_time":       getE(`leave_time_${e}`).value,
        "arrive_time":      getE(`arrive_time_${e}`).value,
        "price":            getE(`price_${e}`).value,
        "capacity":         getE(`capacity_${e}`).value,
        "booked":           getE(`booked_${e}`).value,
    };
    console.log(info)
    
    let data = {
        "type": type,
        "info": info
    }
    console.log(data)

    fetch(`/admin/cflight`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
    }).then((response) => response.json())
    .then((data) => {
        console.log(data);
        if (data["retcode"] == 1) {
            alert("增加成功");
            location.reload();
        } else if (data["retcode"] == 2 ) {
            alert("修改成功");
            location.reload();
        } else if (data["retcode"] ==3 ) {
            alert("删除成功");
            location.reload();
        } else {
            alert("未知错误");
            location.reload();
        }
    })
    .catch((error) => {
        console.error("Error:", error);
        alert("未知错误");
        location.reload();
    });
}

function change_user(e) {
    let data = {
        "uid": getE(`uid_${e}`).value,
        "username": getE(`username_${e}`).value,
        "password": md5(getE(`password_${e}`).value),
        "admin": getE(`admin_${e}`).checked
    };
    console.log(data)

    fetch("/admin/cuser", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
    }).then((response) => response.json())
    .then((data) => {
        console.log(data);
        if (data["retcode"] == 1) {
            alert("用户修改成功");
            location.reload();
        }else if (data["retcode"] == 2) {
            alert("修改成功但未做更改");
            location.reload()
        }else {
            alert("未知错误");
            location.reload();
        }
    })
    .catch((error) => {
        console.error("Error:", error);
        alert("未知错误");
        location.reload();
    });
} 

function buy(e) {
    window.location.href = `/buy?num=${e}`;
}

function pay() {
    let data = {
        uid: getE("uid").innerHTML,
        num: getE("num").innerHTML,
        amount: getE("amount").value,
        time: getE("time").innerHTML,
    };

    fetch("/pay", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data)
    }).then((response) => response.json())
    .then((data) => {
        console.log(data);
        alert(data["message"]);
        history.back();
        setTimeout(function() {
            location.reload();
        }, 1000);
    })
    .catch((error) => {
        console.log(error);
    })
}

drop_pay = (e) => {

    let data = {
        type: 1,
        data: {
            uid: getE("uid").value,
            num: getE(`num_${e}`).innerHTML,
            time: e
        }
    };

    fetch("/user", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data)
    }).then((response) => response.json())
    .then((data) => {
        console.log(data);

        alert(data["message"]);
        location.reload();

    }).catch((error) => {
        console.log(error)
    })
}

user_change = () => {

    let data = {
        type: 2,
        data: {
            username: getE("username").value,
            old_password: md5(getE("old_password").value),
            new_password: md5(getE("new_password").value),
            re_new_password: md5(getE("re_new_password").value),
        }
    };

    fetch("/user", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data)
    }).then((response) => response.json())
    .then((data) => {
        console.log(data);

        alert(data["message"]);
        location.reload();

    }).catch((error) => {
        console.log(error)
    })
}