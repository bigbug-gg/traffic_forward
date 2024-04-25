use crate::{host::{self, ip::{self, Info}}, iptables::{self, tools::{self, Traffic}}};

pub fn list() -> Option<ip::Host> {
    host::ip::host_list()
}

///
/// Add new ip forward
/// 
pub fn add(target_ip: &str, target_port: &str, local_port: &str) -> Result<(), String>{
    let info = Info{
        ip: target_ip.to_string(),
        target_port: target_port.to_string(),
        local_port: local_port.to_string(),
    };

    // First save ip
    if let Err(e) = ip::save_host(info) {
        return Err(e);
    }

    // Then write iptables rule, We needs tcp and udp.
    for i in ["tcp",  "udp"] {
        if let Err(e) = iptables::tools::add(local_port, target_ip, target_port, None, Some(i), None) {
            ip::delete_host(target_ip); 
            return Err(e);
        }
    }
    
    return Ok(());
}

///
/// Delete Forwar
/// 
pub fn del(ip: &str) -> Result<(), String> {
    
    if ip::exists(ip)? {
        ip::delete_host(ip);
    }
    tools::delete(ip)?;
    Ok(())
}

pub fn traffic(ip: &str) -> Result<Traffic, String>{
    if !ip::exists(ip)? {
        return Err("No matching IP found".to_string());
    }

    return tools::traffic(ip)
}