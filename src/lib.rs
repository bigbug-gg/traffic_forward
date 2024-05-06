pub mod host;
pub mod iptables;
pub mod service;
pub mod web;


///
/// Api Server
/// 
/// port
/// 
pub fn api_server(port: u16) {
    let _ = web::run(port);
}

