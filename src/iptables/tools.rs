

use nix::libc::getuid;

use rexpect::spawn;
use std::{io::BufRead, process::{Command}, thread};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prerouting() {
        let pre_str = generate_prerouting_command("2002", "192.168.2.2", "4004", "tcp", None);
        println!("prerouting: {}", pre_str);
        assert_eq!("sudo iptables -t nat -I PREROUTING -p tcp --dport 2002 -j DNAT --to-destination 192.168.2.2:4004", pre_str);
    }
}

///
/// Add new iptables forward
///
pub fn add(
    local_port: &str,
    target_ip: &str,
    target_port: &str,
    comment: Option<&str>,
    protocol: Option<&str>,
    self_ip: Option<&str>,
    user_password: Option<&str>,
) -> Result<(), String> {
    // if not protocol set default tcp
    let _protocl = if protocol.is_none() {
        "tcp"
    } else {
        protocol.unwrap()
    };

    let prerouting =
        generate_prerouting_command(local_port, target_ip, target_port, _protocl, comment);

    let postrouting =
        generate_postrouting_command(target_ip, target_port, _protocl, comment, self_ip);
    if postrouting.is_err() {
        return Err(postrouting.err().unwrap());
    }
    let postrouting = postrouting.unwrap();

    let forward = generate_forward_command(target_ip, target_port, comment, protocol);
    if forward.is_err() {
        return Err(forward.err().unwrap());
    }
    let forward = forward.unwrap();
    let forward_up = forward.up;
    let forward_down = forward.down;

    let command_vec = vec![
        prerouting.as_str(),
        postrouting.as_str(),
        forward_up.as_str(),
        forward_down.as_str(),
    ];

    let mut hands = Vec::new();
    for command in command_vec {
        let a = command.to_owned();
        let p = if user_password.is_none() {
            ""
        } else {
            user_password.unwrap()
        };
        let p = p.to_owned();
        let one = thread::spawn(move || {
            let _ = run_command(&a, Some(&p));
        });
        hands.push(one);
    }

    for h in hands {
        h.join().unwrap();
    }

    Ok(())
}

///
/// Check
///
pub fn traffic(target_ip: &str, sudo_password: Option<&str>) -> Result<(u64, u64), String> {
    let res = run_command(
        "sudo iptables -t filter -vxnL FORWARD --line",
        sudo_password,
    );
    let mut up = 0;
    let mut down = 0;

    for i in res.unwrap() {
        if !i.contains(target_ip) {
            continue;
        }
        let mut m: Vec<&str> = i.split(" ").filter(|_i| !_i.is_empty()).collect();
        let p_str = m.pop().unwrap();
        if target_ip.eq(p_str) {
            up += m.get(1).unwrap().parse::<u64>().unwrap();
        } else {
            down += m.get(1).unwrap().parse::<u64>().unwrap();
        }
    }
    Ok((up, down))
}

///
/// Delete
///
/// get target ip and delete all of forward config with they.
///
pub fn delete(target_ip: &str, sudo_password: Option<&str>) -> Result<(), String> {
    let fn_list = vec![
        |a: &str, p: Option<&str>| delete_forward(a, p),
        |a: &str, p: Option<&str>| delete_postrouting(a, p),
        |a: &str, p: Option<&str>| delete_prerouting(a, p),
    ];

    let mut handles = Vec::new();
    for su_fn in fn_list {
        let a = target_ip.to_owned();
        let p = if sudo_password.is_none() {
            ""
        } else {
            sudo_password.unwrap()
        };
        let p = p.to_owned();

        let h = thread::spawn(move || su_fn(&a, Some(&p)));
        handles.push(h);
    }

    let mut error: String = String::new();
    for h in handles {
        if let Err(e) = h.join().unwrap() {
            error.push_str(&e);
        }
    }

    if error.len() > 0 {
        return Err(error);
    }

    return Ok(());
}

fn delete_prerouting(target_ip: &str, sudo_password: Option<&str>) -> Result<(), String> {
    loop {
        let res = run_command("sudo iptables -t nat -vnL PREROUTING --line", sudo_password);
        let mut line_str: Option<String> = None;
        for i in res.unwrap() {
            if i.contains(target_ip) {
                line_str = Some(i);
                break;
            }
        }

        if line_str.is_none() {
            break;
        }

        let line_row = line_str.unwrap();
        let line_vec: Vec<&str> = line_row.split("    ").filter(|_i| !_i.is_empty()).collect();

        // Get ip index, delete we need this
        let rule_index = line_vec.first().unwrap().trim();
        let _ = run_command(
            format!("sudo iptables -t nat -D PREROUTING {}", rule_index).as_str(),
            sudo_password,
        )
        .unwrap();
    }
    Ok(())
}

fn delete_postrouting(target_ip: &str, sudo_password: Option<&str>) -> Result<(), String> {
    loop {
        let res = run_command(
            "sudo iptables -t nat -vnL POSTROUTING --line",
            sudo_password,
        );
        let mut line_str: Option<String> = None;
        for i in res.unwrap() {
            if i.contains(target_ip) {
                line_str = Some(i);
                break;
            }
        }

        if line_str.is_none() {
            break;
        }

        let line_row = line_str.unwrap();
        let line_vec: Vec<&str> = line_row.split("    ").filter(|_i| !_i.is_empty()).collect();

        // Get ip index, delete we need this
        let rule_index = line_vec.first().unwrap().trim();
        let _ = run_command(
            format!("sudo iptables -t nat -D POSTROUTING {}", rule_index).as_str(),
            sudo_password,
        )
        .unwrap();
    }
    Ok(())
}

fn delete_forward(target_ip: &str, sudo_password: Option<&str>) -> Result<(), String> {
    loop {
        let res = run_command("sudo iptables -t filter -vnL FORWARD --line", sudo_password);
        let mut line_str: Option<String> = None;
        for i in res.unwrap() {
            if i.contains(target_ip) {
                line_str = Some(i);
                break;
            }
        }

        if line_str.is_none() {
            break;
        }

        let line_row = line_str.unwrap();
        let line_vec: Vec<&str> = line_row.split("    ").filter(|_i| !_i.is_empty()).collect();

        // Get ip index, delete we need this
        let rule_index = line_vec.first().unwrap().trim();
        let _ = run_command(
            format!("sudo iptables -t filter -D FORWARD {}", rule_index).as_str(),
            sudo_password,
        )
        .unwrap();
    }
    Ok(())
}

/**
 * Generate Prerouting command with
 */
fn generate_prerouting_command(
    local_port: &str,
    target_ip: &str,
    target_port: &str,
    protocol: &str,
    comment: Option<&str>,
) -> String {
    let mut command_str = String::from("sudo iptables -t nat -I PREROUTING");

    // like: sudo iptables -t nat -I PREROUTING -p tcp --dport 8083
    command_str += format!(" -p {} --dport {}", protocol, local_port).as_str();

    let port_str = format!(" -j DNAT --to-destination {}:{}", target_ip, target_port);

    // like: sudo iptables -t nat -I PREROUTING -p tcp --dport 8083 -j DNAT --to-destination target_ip:target_port
    command_str += port_str.as_str();

    if comment.is_some() {
        // like: sudo iptables -t nat -I PREROUTING -p tcp
        command_str += format!(" -m comment --comment \"{}\"", comment.unwrap()).as_str();
    }

    command_str
}

///
/// Generate Postrouting command
///
fn generate_postrouting_command(
    target_ip: &str,
    target_port: &str,
    protocol: &str,
    comment: Option<&str>,
    self_host_ip: Option<&str>,
) -> Result<String, String> {
    let mut command_str = format!("sudo iptables -t nat -I POSTROUTING -d {}", target_ip);

    // sudo iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port
    command_str += format!(" -p {} --dport {}", protocol, target_port).as_str();

    let self_ip = if self_host_ip.is_none() {
        local_ip().unwrap()
    } else {
        self_host_ip.unwrap().to_string()
    };

    // like: sudo iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port -j SNAT --to-source self_ip
    command_str += format!(" -j SNAT --to-source {}", self_ip).as_str();

    if comment.is_some() {
        // like: sudo iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port -j SNAT --to-source self_ip -m comment --comment "Postrouting"
        command_str += format!(" -m comment --comment \"{}\"", comment.unwrap()).as_str();
    }

    Ok(command_str)
}

///
/// ForwardCommand Type
/// It is necessary to use for traffic statistics
///
#[derive(Debug)]
struct ForwardCommand {
    down: String, // Download traffic
    up: String,   // Upload traffic
}

///
/// Generate Forward command
///
fn generate_forward_command(
    target_ip: &str,
    target_port: &str,
    comment: Option<&str>,
    protocol: Option<&str>,
) -> Result<ForwardCommand, String> {
    let mut command_up_str = format!("sudo iptables -t filter -I FORWARD -d {}", target_ip);
    let mut command_down_str = format!("sudo iptables -t filter -I FORWARD -s {}", target_ip);

    if protocol.is_some() {
        // like: sudo iptables -t nat -I FORWARD -d target_ip -p udp --dport target_port
        command_up_str += format!(
            " -p {} --dport {}",
            protocol.unwrap().to_lowercase(),
            target_port
        )
        .as_str();
        command_down_str += format!(
            " -p {} --dport {}",
            protocol.unwrap().to_lowercase(),
            target_port
        )
        .as_str();
    }

    if comment.is_some() {
        // like: sudo iptables -t nat -I FORWARD -s target_ip -p udp --dport target_port -m comment --comment "..."
        command_up_str += format!(" -m comment --comment \"{}\"", comment.unwrap()).as_str();
        command_down_str += format!(" -m comment --comment \"{}\"", comment.unwrap()).as_str();
    }

    Ok(ForwardCommand {
        down: command_down_str,
        up: command_up_str,
    })
}

///
/// Run command
///
/// If Need password, need set passworld else set  None
///
fn run_command(command: &str, password: Option<&str>) -> Result<Vec<String>, String> {
    return run_command_v2(command, password);
    let mut session = spawn(command, None).unwrap();

    let need_password = session.exp_regex("password");
    if need_password.is_ok() {
        // need password, but not set, exit
        if password.is_none() {
            let _ = session.send_line("exit").unwrap();
            let _ = session.exp_eof().unwrap();
            return Err("Need passwrord".to_string());
        }
        let _ = session.send_line(password.unwrap());
    }

    let mut res = Vec::new();
    loop {
        let i = session.read_line();
        if i.is_err() {
            break;
        }
        res.push(i.unwrap());
    }

    let _ = session.send_line("exit\r").unwrap();
    let _ = session.exp_eof().unwrap();
    Ok(res)
}


pub fn ls() {
    let data = run_command_v2("ls -lh", None);
    if let Ok(d) = data {
        for i in d  {
            println!("{i}");
        }
    }
   
}

fn run_command_v2(command_str: &str, _: Option<&str>) -> Result<Vec<String>, String> {

    if !is_root() {
        panic!("Please use the root account to run");
    }

    let mut command_vec: Vec<&str> = command_str.split(' ').collect();
    if command_vec.len() <= 0 {
        return Err("Please enter a valid command".to_string());
    }

    let mut first_command = String::new();

    loop {
        if !first_command.is_empty() && !first_command.eq("sudo") {
            break;
        }
        first_command = command_vec.first().unwrap().to_string();
        command_vec.remove(0);
    }

    if first_command.is_empty() {
        return Err("No valid instructions detected".to_string())
    }

    let mut res = Vec::new();

    // build and run command.
    let mut comand = Command::new(first_command);
    for arg in command_vec {
        comand.arg(arg);
    }

    let output = comand.output().expect("Can not get data.");
    for line in output.stdout.lines() {
        if let Ok(i) = line {
            res.push(i);
        } else {
            return Err(line.err().unwrap().to_string());
        }
    }
   
    Ok(res)
}

pub fn is_root() -> bool {
    let uid = unsafe { getuid() };
    uid == 0
}

///
/// Get Local IP
///
fn local_ip() -> Option<String> {
    let output = Command::new("hostname")
        .arg("-I")
        .output()
        .expect("can not exec hostname");

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let local_ip: Vec<&str> = stdout.split(' ').collect();

    if local_ip.first().is_none() {
        return None;
    }
    let i = local_ip.first().unwrap().to_string();
    return Some(i);
}
