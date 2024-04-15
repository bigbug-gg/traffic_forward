
pub mod iptables;
use iptables::tools;

fn main() {
    // tools::run_str_command("sudo iptables -nvL");
    let command_str = tools::add("5005", "183.232.232.10", "3333", None, None, None, None);
    println!("Comand is: {:?}", command_str);
}
