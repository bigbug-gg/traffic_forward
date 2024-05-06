# Traffic forward

> 流量转发

*Read this in other languages: [English](README.md).*

# 它能干嘛？

这个就一个便捷工具，帮助编写 iptables 的转发规则，因为正确的书写转发规则需要多条命令，但是，用这个工具，就可以一条命令搞定！

比如添加转发规则:
```bash
sudo traffic_forward add 193.23.11.3:3333 9988
```
等同于:
```bash
iptables -t nat -I PREROUTING -p tcp --dport 9988 -j DNAT --to-destination 193.23.11.3:3333
iptables -t nat -I POSTROUTING -d 193.23.11.3 -p tcp --dport 3333 -j SNAT --to-source 192.168.17.131
iptables -t filter -I FORWARD -d 193.23.11.3 -p tcp --dport 3333
iptables -t filter -I FORWARD -s 193.23.11.3 -p tcp --dport 3333
iptables -t nat -I PREROUTING -p udp --dport 9988 -j DNAT --to-destination 193.23.11.3:3333
iptables -t nat -I POSTROUTING -d 193.23.11.3 -p udp --dport 3333 -j SNAT --to-source 192.168.17.131
iptables -t filter -I FORWARD -d 193.23.11.3 -p udp --dport 3333
iptables -t filter -I FORWARD -s 193.23.11.3 -p udp --dport 3333
```



# 快速上手


## 前提、
* 必须是 Linux 服务器。
* 已经开启了流量转发。【95% 的人都没有做这一步】
* 安装 `Rust`环境。

## 说明

本工具只是代替书写 `iptables` 规则，用一条命令解决多条配置，因为使用到了 `iptables` ,所以需要使用 `root` 账户操作。

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

# 演示用例

``` bash
$ sudo traffic_forward -V
traffic_forward 0.1.2

$ sudo traffic_forward add 192.168.11.11:3389 2233
Add completed

$ sudo traffic_forward list
0.0.0.0:5000 -> 192.145.2.22:323
0.0.0.0:2233 -> 192.168.11.11:3389

$ sudo traffic_forward query 192.168.11.11
Up: 0 KB 
Down: 0 KB

$ sudo traffic_forward delete 192.168.11.11
Delete completed

$ sudo traffic_forward query 192.168.11.11
Query error: No matching IP found

$ sudo traffic_forward list
0.0.0.0:5000 -> 192.145.2.22:323

```

# license 

MIT OR Apache-2.0