use one_test::iptables::tools;

#[test]
fn one_forward() {
    let res = tools::add("5005", "183.232.232.10", "444", None, None, None, Some("root"));
    assert!(res.is_ok(), "error: {}", res.err().unwrap());
}


#[test]
fn del_forward() {
    let res = tools::delete("183.232.232.10", Some("root"));
    assert!(res.is_ok(), "error: {}", res.err().unwrap());
}
#[test]
fn check() {
    let res = tools::check("183.232.232.10", Some("root"));
    assert!(res.is_ok(), "error: {}", res.err().unwrap());
}
