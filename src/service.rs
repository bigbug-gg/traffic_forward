//!
//! The service layer provides a unified invocation method
//!
use crate::{
    host::{
        self,
        ip::{self, Info},
    },
    iptables::{
        self,
        tools::{self, Traffic},
    },
};

pub fn list() -> Option<ip::Host> {
    host::ip::new().list()
}

///  Add new ip forward
pub fn add(target_ip: &str, target_port: &str, local_port: &str) -> Result<(), String> {
    let ip_obj = ip::new();
    let info = Info {
        ip: target_ip.to_string(),
        target_port: target_port.to_string(),
        local_port: local_port.to_string(),
        ..Default::default()
    };

    // First save ip
    if let Err(e) = ip_obj.save(info) {
        return Err(e);
    }

    // Then write iptables rule, We needs tcp and udp.
    for i in ["tcp", "udp"] {
        if let Err(e) =
            iptables::tools::add(local_port, target_ip, target_port, None, Some(i), None)
        {
            ip_obj.delete(target_ip);
            return Err(e);
        }
    }

    return Ok(());
}

/// Delete Forwar
pub fn del(ip: &str) -> Result<(), String> {
    let ip_obj = ip::new();
    if ip_obj.exists(ip)? {
        ip_obj.delete(ip);
    }

    tools::delete(ip)?;
    Ok(())
}

/// Query traffic
pub fn traffic(ip: &str) -> Result<Traffic, String> {
    if !ip::new().exists(ip)? {
        return Err("No matching IP found".to_string());
    }
    return tools::traffic(ip);
}
