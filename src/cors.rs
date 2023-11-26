use rocket::http::{Method};
use rocket_cors::{Cors, AllowedOrigins, AllowedHeaders};


pub fn get_cors() -> Cors {
	// 允许访问的域，这里允许全部，如果要指定其他可以
	// let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);
    let allowed_origins = AllowedOrigins::All;
    // You can also deserialize this
    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Options].into_iter().map(From::from).collect(),
        // 指定header：AllowedHeaders::some(&["Authorization", "Accept"]),
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }.to_cors().expect("cors config error")
}
