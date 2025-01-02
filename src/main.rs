use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let listener = std::net::TcpListener::bind("0.0.0.0:8080").unwrap();
    let server = tokio::spawn(
        actix_web::HttpServer::new(|| actix_web::App::new().route("/", web::get().to(home)))
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
