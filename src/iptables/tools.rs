use nix::libc::getuid;

use std::{fmt::Display, io::BufRead, process::Command, thread};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prerouting() {
        let pre_str = generate_prerouting_command("2002", "192.168.2.2", "4004", "tcp", None);
        println!("prerouting: {}", pre_str);
        assert_eq!("iptables -t nat -I PREROUTING -p tcp --dport 2002 -j DNAT --to-destination 192.168.2.2:4004", pre_str);
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

    println!("{:#?}", command_vec);
    let mut hands = Vec::new();
    for command in command_vec {
        let a = command.to_owned();
        let one = thread::spawn(move || {
            let _ = run_command(&a);
        });
        hands.push(one);
    }

    for h in hands {
        h.join().unwrap();
    }

    Ok(())
}



///
/// Traffic (Unit: byte)
/// 
#[derive(Debug)]
pub struct  Traffic{
    pub up: u64,
    pub down: u64
}

impl Display for Traffic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Up: {} KB \nDown: {} KB", self.up, self.down)
    }
}

///
/// Check 
///
pub fn traffic(target_ip: &str) -> Result<Traffic, String> {
    let res = run_command("iptables -t filter -vxnL FORWARD --line");
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
    Ok(Traffic{
        up,
        down
    })
}

///
/// Delete
///
/// get target ip and delete all of forward config with they.
///
pub fn delete(target_ip: &str) -> Result<(), String> {
    let fn_list = vec![
        |a: &str| delete_forward(a),
        |a: &str| delete_postrouting(a),
        |a: &str| delete_prerouting(a),
    ];

    let mut handles = Vec::new();
    for su_fn in fn_list {
        let a = target_ip.to_owned();
        let h = thread::spawn(move || su_fn(&a));
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

fn delete_prerouting(target_ip: &str) -> Result<(), String> {
    loop {
        let res = run_command("iptables -t nat -vnL PREROUTING --line");
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
            format!("iptables -t nat -D PREROUTING {}", rule_index).as_str()
        )
        .unwrap();
    }
    Ok(())
}

fn delete_postrouting(target_ip: &str) -> Result<(), String> {
    loop {
        let res = run_command(
            "iptables -t nat -vnL POSTROUTING --line"
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
            format!("iptables -t nat -D POSTROUTING {}", rule_index).as_str()
        )
        .unwrap();
    }
    Ok(())
}

fn delete_forward(target_ip: &str) -> Result<(), String> {
    loop {
        let res = run_command("iptables -t filter -vnL FORWARD --line");
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
            format!("iptables -t filter -D FORWARD {}", rule_index).as_str()
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
    let mut command_str = String::from("iptables -t nat -I PREROUTING");

    // like: iptables -t nat -I PREROUTING -p tcp --dport 8083
    command_str += format!(" -p {} --dport {}", protocol, local_port).as_str();

    let port_str = format!(" -j DNAT --to-destination {}:{}", target_ip, target_port);

    // like: iptables -t nat -I PREROUTING -p tcp --dport 8083 -j DNAT --to-destination target_ip:target_port
    command_str += port_str.as_str();

    if comment.is_some() {
        // like: iptables -t nat -I PREROUTING -p tcp
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
    let mut command_str = format!("iptables -t nat -I POSTROUTING -d {}", target_ip);

    // iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port
    command_str += format!(" -p {} --dport {}", protocol, target_port).as_str();

    let self_ip = if self_host_ip.is_none() {
        local_ip().unwrap()
    } else {
        self_host_ip.unwrap().to_string()
    };

    // like: iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port -j SNAT --to-source self_ip
    command_str += format!(" -j SNAT --to-source {}", self_ip).as_str();

    if comment.is_some() {
        // like: iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port -j SNAT --to-source self_ip -m comment --comment "Postrouting"
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
    let mut command_up_str = format!("iptables -t filter -I FORWARD -d {}", target_ip);
    let mut command_down_str = format!("iptables -t filter -I FORWARD -s {}", target_ip);

    if protocol.is_some() {
        // like: iptables -t nat -I FORWARD -d target_ip -p udp --dport target_port
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
        // like: iptables -t nat -I FORWARD -s target_ip -p udp --dport target_port -m comment --comment "..."
        command_up_str += format!(" -m comment --comment \"{}\"", comment.unwrap()).as_str();
        command_down_str += format!(" -m comment --comment \"{}\"", comment.unwrap()).as_str();
    }

    Ok(ForwardCommand {
        down: command_down_str,
        up: command_up_str,
    })
}

///
/// Command fn
/// 
fn run_command(command_str: &str) -> Result<Vec<String>, String> {

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
