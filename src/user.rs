use mysql::{PooledConn, prelude::Queryable};

use crate::initdb;

pub struct User {
    pub uid: i32,
    pub username: String,
    pub password: String,
    pub admin: bool,
}

impl User {
    pub fn user_register(&mut self, conn: &mut PooledConn) -> i32 {
        // 获取最后一名用户uid
        let last_user: Vec<(i32, String, String, bool)> = conn.query(
            "SELECT * FROM users ORDER BY uid DESC LIMIT 1").unwrap();

        let uid: i32 = last_user[0].0 + 1;
        self.uid = uid;

        // 用户名不存在时可以注册
        conn.query_drop(format!("
            INSERT IGNORE INTO users (uid, username, password, admin) VALUES ({}, '{}', '{}', {})",
            self.uid, self.username.as_str(), self.password.as_str(), self.admin
        )).unwrap();

        if conn.affected_rows() > 0 {
            return 1; // 注册成功
        } else {
            return 2; // 用户存在
        }
    }

    pub fn user_login(&mut self, conn: &mut PooledConn) -> i32 {

        let info: Vec<(i32, String, String, bool)> = conn.query(format!("
            SELECT * FROM users WHERE username = '{}'",
            self.username.as_str()
        )).unwrap();
    
        let status_code: i32;
        if info.len() > 0 {
            if info[0].2 == self.password {
                status_code = 1; // 成功
                (
                    self.uid, 
                    self.username, 
                    self.password, 
                    self.admin
                ) = info[0].clone();
            } else {
                status_code = 2; // 密码错误
            }
        } else {
            status_code = -1; // 未知错误
        }
        return status_code;
    }
}