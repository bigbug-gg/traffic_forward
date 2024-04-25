pub mod host;
pub mod iptables;
pub mod service;
pub mod web;


///
/// Api Server
/// 
pub fn api_server() {
    let _ = web::run();
}

