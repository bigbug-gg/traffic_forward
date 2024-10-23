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
    host::ip::host_list()
}

/// Rebuild by the cache data, temp fn. maybe next version delete.
#[deprecated = "This is temp code, next version delete"]
pub fn rebuild() -> Result<(), String> {
    let data_list = list();

    if data_list.is_none() {
        return Ok(());
    }

    let ip_list = data_list.unwrap().list;
    for i in ip_list {
        for agreement in ["tcp", "udp"] {
            if let Err(e) = iptables::tools::add(
                &i.local_port,
                &i.ip,
                &i.target_port,
                None,
                Some(agreement),
                None,
            ) {
                println!("{}: {}", &i.ip, e)
            }
        }
    }
    Ok(())
}

///  Add new ip forward
pub fn add(target_ip: &str, target_port: &str, local_port: &str) -> Result<(), String> {
    let info = Info {
        ip: target_ip.to_string(),
        target_port: target_port.to_string(),
        local_port: local_port.to_string(),
    };

    // First save ip
    if let Err(e) = ip::save_host(info) {
        return Err(e);
    }

    // Then write iptables rule, We needs tcp and udp.
    for i in ["tcp", "udp"] {
        if let Err(e) =
            iptables::tools::add(local_port, target_ip, target_port, None, Some(i), None)
        {
            ip::delete_host(target_ip);
            return Err(e);
        }
    }

    return Ok(());
}

/// Delete Forwar
pub fn del(ip: &str) -> Result<(), String> {
    if ip::exists(ip)? {
        ip::delete_host(ip);
    }
    tools::delete(ip)?;
    Ok(())
}

/// Query traffic
pub fn traffic(ip: &str) -> Result<Traffic, String> {
    if !ip::exists(ip)? {
        return Err("No matching IP found".to_string());
    }

    return tools::traffic(ip);
}
