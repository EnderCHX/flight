use mysql::{PooledConn, prelude::Queryable};

use crate::initdb;

pub struct User {
    pub uid: u32,
    pub username: String,
    pub password: String,
    pub admin: bool,
}

impl User {
    pub fn user_register(&mut self, conn: &mut PooledConn) -> bool{
        let last_user: Vec<(u32, String, String, bool)> = conn.query(
            "SELECT * FROM users ORDER BY uid DESC LIMIT 1").unwrap();
        let uid = last_user[0].0 + 1;
        self.uid = uid;
        conn.query_drop(format!("
            INSERT IGNORE INTO users (uid, username, password, admin) VALUES ({}, '{}', '{}', {})",
            self.uid, self.username.as_str(), self.password.as_str(), self.admin
        )).unwrap();
        if conn.affected_rows() > 0 {
            return true;
        } else {
            return false;
        }
    }
}