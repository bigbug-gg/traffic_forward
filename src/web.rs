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
pub async fn run<'a>(port: u16, token: String) -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let long_time_str = gen_static_str(token);
    info!("Access address: http://localhost:{}", port);
    info!("When making API requests, include a header with the token as a parameter. {}", long_time_str);
   
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(
                web::scope("")
                .guard(guard::Header("token", long_time_str))
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

fn gen_static_str(st: String) ->&'static str{
    Box::leak(st.into_boxed_str())
}