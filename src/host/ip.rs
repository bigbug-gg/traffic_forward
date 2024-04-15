use std::fs::File;
use std::io::Write;


use ron::de::from_reader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Host {
    list: Vec<Info>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub ip: String,
    pub target_port: String,
    pub local_port: String,
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
fn host_list() -> Option<Host> {
    let content = File::open(host_path()).expect("Failed opening file");

    let config: Host = match from_reader(content) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load config: {}", e);
            return None;
        }
    };

    Some(config)
}

///
/// Host Path
///
fn host_path() -> &'static str {
    "src/host/host.ron"
}
