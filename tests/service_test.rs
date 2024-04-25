use traffic_forward::service;


#[test]
fn test_home()
{
    use std::env;
    match env::home_dir() {
        Some(path) => {
            println!("{}", path.display())
        },
        None => println!("Not found"),
    }
} 

#[test]
fn test_forward()
{
    let target_ip = "192.168.50.2";
    let target_port = "8800";
    let local_port = "2002";

    if let Err(e) = service::add(
        target_ip,
        target_port,
        local_port
    ) {
        assert!(false, "Forward Error: {}", e); 
    } else {
        assert!(true); 
    }  
}

#[test]
fn test_delete_forward()
{
    let target_ip = "192.168.50.2";

    if let Err(e) = service::del(
        target_ip
    ) {
        assert!(false, "Forward Error: {}", e); 
    } else {
        assert!(true); 
    }  
}

#[test]
fn test_traffic_forward()
{
    let target_ip = "192.168.50.2";

    let data = service::traffic(
        target_ip
    );

    if let Err(e) = data {
        assert!(false, "Forward Error: {}", e); 
    } else {
        println!("traffic forward: {:?}", data.unwrap());
        assert!(true); 
    }  
}