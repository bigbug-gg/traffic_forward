use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};


pub(crate) mod forward;

///
/// Index default enter
/// 
async fn index() -> impl Responder {
    HttpResponse::Ok().body("RDP-PRO")
}

#[actix_web::main]
pub async fn run() -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(
                forward::enter()
            )
            .route("/", web::get().to(index))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await

}