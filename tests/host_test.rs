use traffic_forward::host::ip::{self, Info, Ip};

fn ip_obj() -> Ip {
    ip::new()
}

#[test]
fn ip_exists() {
    let ip = "182.3.1.22";
    let is_ok = ip_obj().exists(ip);
    println!("The test is: {:?}", is_ok);
}

#[test]
fn ip_save() {
    let data = Info {
        id: None,
        ip: "182.3.1.22".to_string(),
        target_port: "2233".to_string(),
        local_port: "4444".to_string(),
    };
    let is_ok = ip_obj().save(data);

    match is_ok {
        Ok(_) => assert!(true),
        Err(e) => println!("{}", e.to_string()),
    }
}

#[test]
fn ip_list() {
    let is_ok = ip_obj().list();
    match is_ok {
        Some(d) => {
            println!("put ip info:");
            for info in d.list {
                println!("{:?}", info)
            }
        }
        None => println!("No Data"),
    }
}

#[test]
fn ip_delete() {
    let is_ok = ip_obj().delete("182.3.1.22");
    assert!(is_ok)
}
