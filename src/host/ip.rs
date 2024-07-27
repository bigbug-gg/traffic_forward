/**!
 * Ip store logic (CRUD)
 */
use rusqlite::Connection;
use serde::Serialize;
use std::fmt::Display;

/// List of IP addresses for all hosts
#[derive(Debug, Default, Serialize)]
pub struct Host {
    pub list: Vec<Info>,
}

/// Info for IP
#[derive(Debug, Default, Serialize)]
pub struct Info {
    /// id
    pub id: Option<i32>,

    /// forward ip
    pub ip: String,

    /// forward port
    pub target_port: String,

    /// Traffic forwarding host port
    pub local_port: String,
}

// const TABLE_NAME: &str = "forward_table";

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.list.len() <= 0 {
            write!(f, "No Data")?;
        }

        for i in &self.list {
            write!(
                f,
                "0.0.0.0:{} -> {}:{}\n",
                i.local_port, i.ip, i.target_port
            )?;
        }

        Ok(())
    }
}

/// Ip struct
/// Container a sqlite connection object.
pub struct Ip {
    conn: Connection,
}
pub fn new() -> Ip {
    Ip { conn: get_conn() }
}

impl Ip {
    /// Check ip if real exists
    pub fn exists(&self, ip: &str) -> Result<bool, String> {
        let r: Result<usize, rusqlite::Error> = self.conn.query_row(
            "SELECT COUNT(*) from forward_table WHERE ip = ?1 ",
            [ip],
            |row| row.get(0),
        );

        if let Err(e) = r {
            return Err(e.to_string());
        }

        let r: usize = r.unwrap();
        if r > 0 {
            return Ok(true);
        }

        return Ok(false);
    }

    /// Check port
    pub fn exists_local_port(&self, port: &str) -> Result<bool, String> {
        let r: Result<usize, rusqlite::Error> = self.conn.query_row(
            "SELECT COUNT(*) from forward_table WHERE local_port = ?1 ",
            [port],
            |row| row.get(0),
        );

        if let Err(e) = r {
            return Err(e.to_string());
        }

        let r: usize = r.unwrap();
        if r > 0 {
            return Ok(true);
        }

        return Ok(false);
    }

    /// Save Target Host
    pub fn save(&self, info: Info) -> Result<(), String> {
        if self.exists(&info.ip)? {
            return Err(String::from("IP already exists"));
        }

        if self.exists_local_port(&info.local_port)? {
            return Err(String::from(
                "The transit host port has been used(local_port)",
            ));
        }

        self.conn
            .execute(
                "
        INSERT INTO forward_table (ip, target_port, local_port) VALUES (?1, ?2, ?3)
        ",
                (&info.ip, &info.target_port, &info.local_port),
            )
            .unwrap();

        return Ok(());
    }

    /// Delete Host
    pub fn delete(&self, ip: &str) -> bool {
        let one = self
            .conn
            .execute("DELETE FROM forward_table WHERE ip = ?1;", [ip]);

        if let Err(e) = one {
            print!("error is: {}", e.to_string());
            return false;
        }

        return true;
    }

    /// Get All Target Host Info
    pub fn list(&self) -> Option<Host> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, ip, target_port, local_port FROM forward_table")
            .unwrap();

        let mut host = Host::default();

        let mut q = stmt.query([]).unwrap();

        while let Some(row) = q.next().unwrap() {
            let info = Info {
                id: row.get(0).unwrap(),
                ip: row.get(1).unwrap(),
                target_port: row.get(2).unwrap(),
                local_port: row.get(3).unwrap(),
            };
            host.list.push(info);
        }

        Some(host)
    }
}

/// Host Path
fn db_path() -> String {
    "/etc/traffic_forward.db".to_string()
}

fn get_conn() -> Connection {
    let conn = Connection::open(db_path()).unwrap();
    init_table(&conn);
    conn
}

fn init_table(conn: &Connection) {
    let sql = r#"
    CREATE TABLE IF NOT EXISTS forward_table (
    id INTEGER PRIMARY KEY,
    ip TEXT NOT NULL,
    target_port TEXT NOT NULL,
    local_port TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);
"#;
    conn.execute(sql, []).unwrap();
}
