use traffic_forward::iptables::tools;

#[test]
fn one_forward() {
    let res = tools::add(
        "5005",
        "183.232.232.10",
        "444",
        None,
        None,
        None,
    );
    assert!(res.is_ok(), "error: {}", res.err().unwrap());
}


#[test]
fn del_forward() {
    let res = tools::delete("183.232.232.10");
    assert!(res.is_ok(), "error: {}", res.err().unwrap());
}

#[test]
fn test_traffic() {
    let res = tools::traffic("183.232.232.10");
    println!("{:?}", res);
    assert!(res.is_ok(), "error: {}", res.err().unwrap());
}
