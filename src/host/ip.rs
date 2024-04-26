#[allow(dead_code)]
use std::fs::File;
use std::{env, fmt::Display, io::Write};
use ron::de::from_reader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Host {
    pub list: Vec<Info>,
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        if self.list.len() <= 0 {
            write!(f, "No Data")?;    
        }

        for i in &self.list {
            write!(f, "0.0.0.0:{} -> {}:{}\n", i.local_port, i.ip, i.target_port)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Info {
    pub ip: String,
    pub target_port: String,
    pub local_port: String,
}

///
/// Exists check ip if real in file
/// 
pub fn exists(ip: &str) -> Result<bool, String> {
    let content = host_list();
    if content.is_none() {
        return Err("Fetch Data error!".to_string());
    }

    let ip_host = content.unwrap();
    
    for info in ip_host.list {
        if info.ip.eq(ip) {
            return Ok(true);
        }
    }
    return Ok(false);
}

///
/// Save Target Host
///
pub fn save_host(info: Info) -> Result<(), String> {
    let file_content = host_list();
    let mut host: Host;
    if !file_content.is_none() {
        host = file_content.unwrap();
        let host_exist = host.list.iter().position(|i| i.ip == info.ip);

        if !host_exist.is_none() {
            return Err(String::from("IP already exists"));
        }

        let local_port_exist = host
            .list
            .iter()
            .position(|i| i.local_port == info.local_port);

        if !local_port_exist.is_none() {
            return Err(String::from(
                "The transit host port has been used(local_port)",
            ));
        }
        host.list.push(info);
    } else {
        host = Host { list: vec![info] }
    }

    let need_save = ron::to_string(&host).unwrap();
    let need_save = need_save.as_bytes();
    let mut wirte_file = File::create(host_path()).expect("Can not open file");
    let result = wirte_file.write(&need_save);

    if result.is_err() {
        return Err(result.err().unwrap().to_string());
    }

    return Ok(());
}

///
/// Delete Host
///
pub fn delete_host(ip: &str) {
    let file_content = host_list();

    if file_content.is_none() {
        return;
    }

    let mut host = file_content.unwrap();
    let host_exist = host.list.iter().position(|i| i.ip == ip);

    if host_exist.is_none() {
        return;
    }

    let index = host_exist.unwrap();
    host.list.remove(index);

    let need_save = ron::to_string(&host).unwrap();
    let need_save = need_save.as_bytes();
    let mut wirte_file = File::create(host_path()).expect("Can not open file");
    let _ = wirte_file.write(&need_save);
}

///
/// Get All Target Host Info
///
pub fn host_list() -> Option<Host> {

    let mut data = Host::default();
    if let Ok(content) = File::open(host_path()) {
         data = match from_reader(content) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to load config: {}", e);
                return None;
            }
        };
    }
    
    Some(data)
}

///
/// Host Path
///
fn host_path() -> String {
    match env::home_dir() {
        Some(path) => {
            format!("{}/.traffic_forward.ron", path.display())
        },
        None => panic!(""),
    }
}