//值是否为空
function ifnull(id) {
    if(document.getElementById(id).value==''){
        return "null";
    } else {
        return document.getElementById(id).value;
    }
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
            document.getElementById("info").innerHTML = "注册成功，正在跳转";
            let hash = md5(password); //备用"skOR7oRda5iypO1ejFnmyd2MkDOlYUHG0STBMDBDTXo="
            console.log(hash);
            redrict = `http://${window.location.host}/register?username=${username}&password=${hash}`;
            console.log(redrict);
            window.location.href = redrict;
        } else {
            document.getElementById("info").innerHTML = "密码不一致";
        }
    }
    
}