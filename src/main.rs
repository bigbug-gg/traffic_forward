use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use iptables_forward::service;
use serde::{Deserialize, Serialize};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("RDP-PRO")
}

#[get("")]
async fn iptables() -> impl Responder {
    HttpResponse::Ok().body("RDP-PRO")
}

#[post("/add")]
async fn add_iptables(info: web::Json<AddIptable>) -> Result<impl Responder> {
    let data = service::add(&info.target_ip, &info.target_port, &info.local_port, Some(&info.user_password));

    if let Err(e) = data {
        Ok(JsonData::response(Some(&e),None))
    } else {
        Ok(JsonData::response(None, Some(())))
    }
}

#[post("/del")]
async fn del_iptables(info: web::Json<IptableIp>) ->Result<impl Responder> {

    let data = service::del(&info.target_ip, Some(&info.user_password) );

    if let Err(e) = data {
        Ok(JsonData::response(Some(&e),None))
    } else {
        Ok(JsonData::response(None, Some(())))
    }
}

#[get("/list")]
async fn list_iptables() -> Result<impl Responder> {
    let data = service::list();
    Ok(JsonData::response(None, data))
}

#[derive(Deserialize)]
struct AddIptable {
    target_ip: String, 
    target_port: String, 
    local_port: String, 
    user_password: String,
}

#[derive(Deserialize)]
struct IptableIp {
    target_ip: String, 
    user_password: String,
}

#[derive(Serialize)]
struct JsonData<T> {
    code: i32,
    msg: String,
    data: Option<T>
}

impl<T:Serialize> JsonData<T> {

    pub fn response(msg: Option<&str>, data:Option<T>) -> web::Json<Self> {
        let res = if let Some(d) = data {
            JsonData::success(msg, Some(d))
        } else {
            JsonData::fail(msg, None)
        };
        web::Json(res)
    }

    pub fn fail(msg: Option<&str>, data: Option<T>) -> Self {
        let msg = if let Some(msg_str) = msg {
            msg_str.to_string()
        } else {
            "Fail".to_string()
        };

        JsonData {
            code: 0,
            msg,
            data,
        }
    }

    pub fn success(msg: Option<&str>, data: Option<T>) -> Self {
        let msg = if let Some(msg_str) = msg {
            msg_str.to_string()
        } else {
            "Success".to_string()
        };

        JsonData {
            code: 1,
            msg,
            data,
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/iptables")
                .service(iptables)
                .service(add_iptables)
                .service(del_iptables)
                .service(list_iptables)
            )
            .route("/", web::get().to(index))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}