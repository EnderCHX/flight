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
    pub fn conn(&self, p: bool) -> Result<PooledConn, mysql::Error> {
        let url = format!("mysql://{}:{}@{}:{}", self.user, self.password, self.ip, self.port);
        // 建立数据库连接
        let pool = Pool::new(url.as_str())?;
        // 获取数据库连接
        let conn = pool.get_conn()?;
        if p {
            println!("Connected to database");
        }
        Ok(conn)
    }


    pub fn init_db(&self) {
        let mut conn = self.conn(true).unwrap();
        conn.query_drop(format!("CREATE DATABASE IF NOT EXISTS {}", self.db_name))
            .expect("Failed to create database");
        conn.query_drop(format!("USE {}", self.db_name)).unwrap();
        conn.query_drop(
            r"
            CREATE TABLE IF NOT EXISTS users (
                uid INT auto_increment NOT NULL,
                username VARCHAR(20) UNIQUE NOT NULL,
                password VARCHAR(128) NOT NULL,
                admin int NOT NULL,
                PRIMARY KEY (uid)
            );
            INSERT IGNORE INTO users VALUES (1, 'admin', '21232f297a57a5a743894a0e4a801fc3', 1)
            ",
        ).expect("Failed to create users table");
        conn.query_drop(
            r"
            CREATE TABLE IF NOT EXISTS flights (
                num INT auto_increment NOT NULL,
                leave_city VARCHAR(128) NOT NULL,
                arrive_city VARCHAR(128) NOT NULL,
                leave_airport VARCHAR(128) NOT NULL,
                arrive_airport VARCHAR(128) NOT NULL,
                leave_time BIGINT NOT NULL,
                arrive_time BIGINT NOT NULL,
                price DOUBLE NOT NULL,
                capacity INT NOT NULL,
                booked INT NOT NULL,
                PRIMARY KEY (num)
            )
            ",
        ).expect("Failed to create flights table");

        conn.query_drop(
            r"
            CREATE TABLE IF NOT EXISTS payments (
                uid INT NOT NULL,
                num INT NOT NULL,
                amount INT NOT NULL,
                time BIGINT NOT NULL
            )
            ",
        ).expect("Failed to create payments table");
        drop(conn)
    }
}


