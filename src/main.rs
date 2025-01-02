mod configuration;

use actix_session::config::PersistentSession;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use configuration::Settings;
use redis::{Client, Commands};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let config = configuration::get_configuration().unwrap();
    let redis_store = match RedisSessionStore::new(config.clone().redis_uri).await {
        Ok(store) => store,
        Err(e) => panic!("Error creating Redis session store: {}", e),
    };
    let secret_key = Key::from(config.application.hmac_secret.as_bytes());

    let listener = std::net::TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))
    .unwrap();
    let server = tokio::spawn(
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(
                    SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                        .session_lifecycle(
                            PersistentSession::default().session_ttl(Duration::seconds(3600)),
                        )
                        .cookie_name("zero2prod".to_string())
                        .cookie_secure(
                            std::env::var("COOKIE_SECURE").unwrap_or("false".to_owned()) == "true",
                        )
                        .cookie_path("/".to_string())
                        .build(),
                )
                .route("/", web::get().to(home))
                .route("/redis", web::get().to(test_redis))
                .app_data(web::Data::new(config.clone()))
        })
        .listen(listener)
        .map_err(|e| {
            println!("Error: {}", e);
        })
        .unwrap()
        .run(),
    );

    tokio::select! {
        o = server => {
            println!("Server stopped: {:?}", o);
        }
    }
    ()
}

pub async fn home() -> HttpResponse {
    println!("Home accessed");
    HttpResponse::Ok().content_type(ContentType::html()).body(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Home</title>
    </head>
    <body>
        <p>Hello world!</p>
    </body>
</html>"#
            .to_string(),
    )
}

async fn test_redis(config: Data<Settings>) -> HttpResponse {
    match test_redis_connection(config) {
        Ok(_) => HttpResponse::Ok().body("Redis connection successful!"),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

fn test_redis_connection(config: Data<Settings>) -> Result<(), String> {
    match Client::open(config.redis_uri.as_str()) {
        Ok(client) => match client.get_connection() {
            Ok(mut connection) => {
                connection.set::<&str, &str, ()>("test", "test").unwrap();
                connection.get::<&str, String>("test").unwrap();
                Ok(())
            }
            Err(e) => Err(format!("Error creating Redis connection: {}", e)),
        },
        Err(e) => Err(format!("Error creating Redis client: {}", e)),
    }
}
