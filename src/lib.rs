pub mod host;
pub mod iptables;
pub mod service;
pub mod web;

///
/// enter
/// 
pub fn run() {
    println!("Hi, iptables_forward");
}

///
/// Api Server
/// 
pub fn api_server() {
    let _ = web::run();
}

