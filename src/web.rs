//! Web
//! 
//! Provide web interfaces for external calls

use actix_web::{guard, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use log::info;


pub(crate) mod forward;

/// Index default enter
async fn index() -> impl Responder {
    HttpResponse::Ok().body("RDP-PRO")
}

/// Run
#[actix_web::main]
pub async fn run<'a>(port: u16, _token: &'a str) -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    info!("Access address: http://localhost:{}", port);
    info!("Request must carray heard key token, The value: {}", _token);

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(
                web::scope("/")
                // .guard(guard::Header("token",))
                .service(
                    forward::enter()
                )
            )
            .route("/", web::get().to(index))
           
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}