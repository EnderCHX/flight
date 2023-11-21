// let url = "mysql://root:chxMIMA@@localhost:3306/";
//     let db_name = "db_flight";

//     // 建立数据库连接
//     let pool = Pool::new(url).expect("Failed to create database connection pool");
//     // 获取数据库连接
//     let mut conn = pool.get_conn().expect("Failed to get database connection");

//     // 创建数据库
//     conn.query_drop(format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
//         .expect("Failed to create database");
//     // 切换到新创建的数据库
//     conn.query_drop(format!("USE {}", db_name))
//         .expect("Failed to switch to database");
//     println!("Connected to database: {}", db_name);

//     conn.query_drop(
//         r"
//         CREATE TABLE IF NOT EXISTS users (
//             uid INT UNSIGNED auto_increment NOT NULL,
//             nickname VARCHAR(255) NOT NULL,
//             admin BOOL NOT NULL,
//             PRIMARY KEY (uid)
//         )
//         ",
//     )
//     .expect("Failed to create users table");

//     conn.query_drop(format!("
//         INSERT INTO users
//         VALUES (3, false, admin)
//     ")).expect("failed");

use mysql::*;
use mysql::prelude::*;

pub struct DbInfo {
    pub ip: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
}
impl DbInfo {
    pub fn conn(&self) -> Result<PooledConn, mysql::Error> {
        let url = format!("mysql://{}:{}@{}:{}", self.user, self.password, self.ip, self.port);
        // 建立数据库连接
        let pool = Pool::new(url.as_str())?;
        // 获取数据库连接
        let conn = pool.get_conn()?;
        println!("Connected to database");
        Ok(conn)
    }

    pub fn init_db(&self) {
        let mut conn = self.conn().unwrap();
        conn.query_drop(format!("CREATE DATABASE IF NOT EXISTS {}", self.db_name))
            .expect("Failed to create database");
    }
}


