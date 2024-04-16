
use std::{io::{Read, Write}, process::{Command, Stdio}};

fn main() {
    // 创建子进程并获取其输出流
    let mut child = Command::new("sudo")
        .arg("iptables")
        .arg("-nvL")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    // 向子进程发送密码
    let password = "root";
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(password.as_bytes())
        .unwrap();

    child.stdin.as_mut().unwrap().flush().unwrap();

    // 读取子进程的输出并进行处理
    let output = child.stdout.as_mut().unwrap();
    let mut buffer = String::new();
    output.read_to_string(&mut buffer).unwrap();

    // 在这里对输出进行处理，例如解析、提取信息等

    // 关闭子进程的输出流
    let _ = drop(output);

    // 等待子进程结束并获取退出状态
    let status = child.wait().unwrap();
    println!("{:?}", status);
}
