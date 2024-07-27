//! Sealing of conversion methods

use nix::libc::getuid;

use std::{fmt::Display, io::BufRead, process::Command};

/// Add new iptables forward
///
/// Create three iptables commands for chains: prerouting, posting, and forward
pub fn add(
    local_port: &str,
    target_ip: &str,
    target_port: &str,
    comment: Option<&str>,
    protocol: Option<&str>,
    self_ip: Option<&str>,
) -> Result<(), String> {
    // protocol set default tcp
    let protocl = protocol.unwrap_or("tcp");

    let prerouting =
        generate_prerouting_command(local_port, target_ip, target_port, protocl, comment);

    let postrouting =
        generate_postrouting_command(target_ip, target_port, protocl, comment, self_ip);

    let forward = generate_forward_command(target_ip, target_port, comment, protocol);
    if forward.is_err() {
        return Err(forward.err().expect("Generate forward command faild"));
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

    for command in command_vec {
        if let Err(err) = run_command(command) {
            return Err(err);
        }
    }

    Ok(())
}

/// Traffic (Unit: byte)
#[derive(Debug)]
pub struct Traffic {
    /// Uplink traffic(Unit:byte)
    pub up: u64,

    /// Downward traffic(Unit:byte)
    pub down: u64,
}

impl Display for Traffic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Up: {} byte \nDown: {} byte", self.up, self.down)
    }
}

/// Traffic
///
/// Query forwarded upstream and downstream traffic
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
    Ok(Traffic { up, down })
}

/// Delete
///
/// When deleting rules, it is necessary to delete all configurations on the pre routing, posting, and forward chains
pub fn delete(target_ip: &str) -> Result<(), String> {
    // inner fn
    let delete_chain_list = vec![
        ChainType::FORWARD,
        ChainType::POSTROUTING,
        ChainType::PREROUTING,
    ];

    for chain_type in delete_chain_list {
        ip_delete(target_ip, chain_type)?;
    }

    return Ok(());
}

/// iptables Chain Type
#[derive(PartialEq)]
enum ChainType {
    FORWARD,
    POSTROUTING,
    PREROUTING,
}

impl Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ChainType::FORWARD => String::from("FORWARD"),
            ChainType::POSTROUTING => String::from("POSTROUTING"),
            ChainType::PREROUTING => String::from("PREROUTING"),
        };
        write!(f, "{name}")
    }
}

/// Delete IP
fn ip_delete(target_ip: &str, chain_type: ChainType) -> Result<(), String> {
    // There are multiple rules in the configuration, and after deleting one rule,
    // the index of the configuration will also change. Only a loop can delete all configuration items cleanly
    loop {
        let inner_table = if chain_type == ChainType::FORWARD {
            "filter"
        } else {
            "nat"
        };
        let res = run_command(&format!(
            "iptables -t {} -vnL {} --line",
            inner_table, chain_type
        ));
        let mut line_str: Option<String> = None;
        // Fetch index of config in iptables
        for i in res.unwrap() {
            if i.contains(target_ip) {
                line_str = Some(i);
                break;
            }
        }

        if line_str.is_none() {
            break;
        }

        let line_row = line_str.expect("Failed to obtain iptables configuration");
        let line_vec: Vec<&str> = line_row.split(" ").filter(|_i| !_i.is_empty()).collect();

        // Get ip index, delete we need this
        let rule_index = line_vec.first().unwrap().trim();
        run_command(&format!(
            "iptables -t {} -D {} {}",
            inner_table, chain_type, rule_index
        ))?;
    }
    Ok(())
}

/// Generate Prerouting command with
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

/// Generate Postrouting command
fn generate_postrouting_command(
    target_ip: &str,
    target_port: &str,
    protocol: &str,
    comment: Option<&str>,
    self_host_ip: Option<&str>,
) -> String {
    let mut command_str = format!("iptables -t nat -I POSTROUTING -d {}", target_ip);

    // iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port
    command_str += format!(" -p {} --dport {}", protocol, target_port).as_str();

    // Local host ip address
    let owner_host_ip = self_host_ip
        .map(|ip| ip.to_string())
        .or_else(|| local_ip())
        .expect("Failed to get IP address");

    // like: iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port -j SNAT --to-source self_ip
    command_str += format!(" -j SNAT --to-source {}", owner_host_ip).as_str();

    if comment.is_some() {
        // like: iptables -t nat -A POSTROUTING -d target_ip -p udp --dport target_port -j SNAT --to-source self_ip -m comment --comment "Postrouting"
        command_str += format!(" -m comment --comment \"{}\"", comment.unwrap()).as_str();
    }

    command_str
}

/// ForwardCommand Type
/// It is necessary to use for traffic statistics
#[derive(Debug)]
struct ForwardCommand {
    down: String, // Download traffic
    up: String,   // Upload traffic
}

/// Generate Forward command
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

/// The main entry point for calling system commands
fn run_command(command_str: &str) -> Result<Vec<String>, String> {
    let mut command_vec: Vec<&str> = command_str.split(' ').collect();

    if command_vec.len() <= 0 {
        return Err("Please enter a valid command".to_string());
    }

    let program = command_vec.first().unwrap().to_string();
    command_vec.remove(0);

    let data = Command::new(program)
        .args(command_vec)
        .output()
        .expect(&format!("Command call failed: {}", command_str.to_string()));

    if !data.status.success() {
        let mut error_msg = match String::from_utf8(data.stderr) {
            Ok(msg) => msg,
            Err(e) => e.to_string(),
        };

        if !is_root() {
            error_msg.push_str("Permission denied (you must be root)")
        }
        return Err(error_msg);
    }

    let res = data.stdout.lines().map(|s| s.unwrap()).collect();
    Ok(res)
}

/// Is it currently a root account
pub fn is_root() -> bool {
    let uid = unsafe { getuid() };
    uid == 0
}

/// Get Local IP
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
