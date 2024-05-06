use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use log::info;


pub(crate) mod forward;

///
/// Index default enter
/// 
async fn index() -> impl Responder {
    HttpResponse::Ok().body("RDP-PRO")
}

/// Run
/// 
/// Specify web access port
/// 
#[actix_web::main]
pub async fn run(port: u16) -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
   
    info!("Access address: http://localhost:{}", port);
    HttpServer::new(|| {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(
                forward::enter()
            )
            .route("/", web::get().to(index))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await

}