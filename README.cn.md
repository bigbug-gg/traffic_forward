# Traffic forward

> 流量转发

*Read this in other languages: [English](README.md).*
# 快速上手

前提： 服务器已经有 `Rust` 环境。

## 安装:

```bash
cargo install traffic_forward
```

安装成功会看到路径：
```bash
...
 Compiling clap v4.5.4
   Compiling ron v0.8.1
   Compiling traffic_forward v0.1.0
    Finished release [optimized] target(s) in 2m 17s
  Installing /home/youre_account/.cargo/bin/traffic_forward
   Installed package `traffic_forward v0.1.0` (executable `traffic_forward`)
```

【非 root 账户】 添加软链接到 `/usr/bin` ：

```
sudo ln -s /home/youre_account/.cargo/bin/traffic_forward /usr/bin/traffic_forward
```

查看版本：
``` bash
traffic_forward --version
traffic_forward 0.1.0
```

或者：
``` bash
sudo traffic_forward --version
traffic_forward 0.1.0
```

---

## 使用：

* 添加转发：

```bash
# 使用本机 5555 端口，转发 192.102.11.44:8000
sudo traffic_forward add 192.102.11.44:8000 5555
```

* 已有规则列表：
```
sudo traffic_forward list
0.0.0.0:5555 -> 192.102.11.44:8000
```

* 查询指定 ip 消耗流量:
```
sudo traffic_forward query 192.102.11.44
```

* 删除指定 ip 转发规则:
```bash
sudo traffic_forward delete 192.102.11.44
Delete completed
```

* 开启 web API:
```bash
 sudo traffic_forward web
```


---

## web API 接口

添加
* uri: iptables/add
* method: post
* request
```
{
	"target_ip": "192.168.50.50",
	"target_port": "4488",
	"local_port": "4433",
}
```

* response
```
{
	"code": 1,
	"msg": "Success",
	"data": null
}
```

删除
* uri: iptables/del
* method: post
* request
```
{
	"target_ip": "192.168.50.50"
}
```

* response
```
{
	"code": 1,
	"msg": "Success",
	"data": null
}
```


列表
* uri: iptables/list
* method: get
* request: empty (change next version will) 
* response
```
{
	"code": 1,
	"msg": "Success",
	"data": {
		"list": [
			{
				"ip": "192.168.50.50",
				"target_port": "4488",
				"local_port": "4433"
			}
		]
	}
}
```

---

# 写入规则成功，但无法正常使用端口实现流量代理，请确保 Linux 已开启转发