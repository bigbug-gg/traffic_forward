# iptables_forward

> Enable traffic redirection through Linux iptables using a Web API.

# Usage

Deployed on the forwarding server, it already provides the most basic functions: adding forwarding, deleting forwarding, and viewing the forwarding list. If the current functionality does not meet your requirements, please customize it according to your actual needs.
1. Clone:

```
git clone https://github.com/bigbug-gg/iptables_forward.git

cd iptables_forward
```

2. Run 

```
# build Or Install if you want.
cargo run
```

# Api

## Default Path:

0.0.0.0:8080

## add forward rule
* uri: iptables/add
* method: post
* request
```
{
	"target_ip": "192.168.50.50",
	"target_port": "4488",
	"local_port": "4433",
	"user_password": "sudo_password"
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

Generation rules:

```bash
root@bigbug-gg:/# iptables -t nat -nvL
Chain PREROUTING (policy ACCEPT 0 packets, 0 bytes)
 pkts bytes target     prot opt in     out     source               destination         
    0     0 DNAT       udp  --  *      *       0.0.0.0/0            0.0.0.0/0            udp dpt:4433 to:192.168.50.50:4488
    0     0 DNAT       tcp  --  *      *       0.0.0.0/0            0.0.0.0/0            tcp dpt:4433 to:192.168.50.50:4488

Chain INPUT (policy ACCEPT 0 packets, 0 bytes)
 pkts bytes target     prot opt in     out     source               destination         

Chain OUTPUT (policy ACCEPT 0 packets, 0 bytes)
 pkts bytes target     prot opt in     out     source               destination         

Chain POSTROUTING (policy ACCEPT 0 packets, 0 bytes)
 pkts bytes target     prot opt in     out     source               destination         
    0     0 SNAT       udp  --  *      *       0.0.0.0/0            192.168.50.50        udp dpt:4488 to:192.168.17.131
    0     0 SNAT       tcp  --  *      *       0.0.0.0/0            192.168.50.50        tcp dpt:4488 to:192.168.17.131

root@bigbug-gg:/# iptables -t filter -vnL FORWARD
Chain FORWARD (policy ACCEPT 0 packets, 0 bytes)
 pkts bytes target     prot opt in     out     source               destination         
    0     0            udp  --  *      *       192.168.50.50        0.0.0.0/0            udp dpt:4488
    0     0            udp  --  *      *       0.0.0.0/0            192.168.50.50        udp dpt:4488
    0     0            tcp  --  *      *       0.0.0.0/0            192.168.50.50        tcp dpt:4488
    0     0            tcp  --  *      *       192.168.50.50        0.0.0.0/0            tcp dpt:4488

```

## delete forward rule
* uri: iptables/del
* method: post
* request
```
{
	"target_ip": "192.168.50.50",
	"user_password": "sudo_password"
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

## List of forward rule
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