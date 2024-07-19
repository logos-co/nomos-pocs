# Large number of direct UDP connections test

In Nomos DA, the Executor has to maintain persistent connections to 4096 (or more for redundancy) DA Nodes for dispersing the encoded blobs. To see if such number of connections is feasable, a test mimicking high speed dispersal was conducted.

## Test setup

Rudimentary client and server applications was created using s2n-quic crate.

* Client
  - Mimicks behaviour of Zone Executor by sending 1024 bytes packet per connection every second;
  - Opens 10 new udp socket connections every second to the Server;
  - Tracks how many currently open connections there are;
  - Tracks the shortest, longest and average time it takes for the packet to be echoed back from the server.

* Server
  - Mimicks behaviour of Multiple DA Nodes by accepting any incomming udp connections and echoing any data that is received;
  - Tracks total number of currently open connections;
  - Tracks total number of received bytes.

### Hardware used for tests

* Client
  1. (MAC, Kaunas) MacBook Air (Sonoma 14.3.1) 8GB
  2. (PC, Kaunas) i7-4770 CPU @ 3.40GHz (Linux 6.9.8-arch1-1) 32GB
  3. (DC Dedicated, Helsinki) AMD Ryzen 5 3600 6-Core (Linux 5.15.0-88-generic) 64GB

* Server
  1. OVH VPS (Warsaw) vps2020-starter-1-2-20 vCores 1 (Linux 5.10.0-31-cloud-amd64) 2GB

* Network hardware
  1. Router (Technicolor DGA0122)
  2. Switch (TP Link)
  3. 4G AP (Samsung A53)

* Network topologies
  1. MAC Wifi > Router > Server
  2. MAC Wifi > 4G AP > Server
  3. MAC Eth > Router > Server
  4. PC > Switch > Router > Server
  5. PC > Router > Server
  6. DC Dedicated > Server

## Results

1. MAC Wifi > Router > Server ([Client logs](results/client_mac_wifi_router_server.log), [Server logs](results/server_mac_wifi_router_server.log))
  - Issues started appearing around **3349** active connections. Existing connections started failing, new connections was still being created without issues

2. MAC Wifi > 4G AP > Server ([Client logs](results/client_mac_wifi_4gap_server.log), [Server logs](results/server_mac_wifi_4gap_server.log))
  - Issues started appearing around **502** active connections. No new connections were allowed to be created, most likely hard limit by the AP (Samsung A53 Android phone) 

3. MAC Eth > Router > Server ([Client logs](results/client_mac_eth_router_server.log), [Server logs](results/server_mac_eth_router_server.log))
  - Issues started appearing around **5100** active connections. No new connections were allowd to be created because of the hard limit of open files on Mac OS.

4. PC > Switch > Router > Server ([Client logs](results/client_pc_switch_router_server.log), [Server logs](results/server_pc_switch_router_server.log))
  - Issues started appearing around **1598** active connections. Suspected reason physical limits of cheap TP Link switch.

5. PC > Router > Server ([Client logs](results/client_pc_router_server.log), [Server logs](results/server_pc_router_server.log))
  - Issues started appearing around **6267** active connections. Suspected reason physical limits on the network card, unoptimal interface configuration.

6. DC Dedicated > Server ([Client logs](results/client_dc_server.log), [Server logs](results/server_dc_server.log))
  - Issues started appearing around **7258** active connections. Suspected reason DDOS protection on the OVH VPS Server side (got an email about that).

## Conclusions

Having an executor running on consumer level hardware might pose some challenges, but is possible, from a perspective of maintaining large number of connections to large number of remote hosts. Datacenter level machine should be able to handle these connections without complex network configuration. Ideally, the Executor would have couple network interfaces to help spread the load.

## How to run

If server is being deployed on a remote machine, a new certificate needs to be created with updated ip address and the hostname. Follow "Certificate" section for that. Once a new certificate is created, recompile client and server.

## Certificate

To genereate a new key and certificate for remote testing:

```bash
openssl ecparam -name prime256v1 -genkey -noout -out key.pem
openssl req -new -key key.pem -out cert.csr -config san.cnf
openssl x509 -req -in cert.csr -signkey key.pem -out cert.pem -days 365 -extensions req_ext -extfile san.cnf
```

## Cross compilation

To crosscompile to x86 linux target use `x86_64-unknown-linux-musl`, gcc-10 is required for `aws-lc-sys` and as of 2024-07-18, `cross` doens't have this version when using docker.

```bash
cross build --target x86_64-unknown-linux-musl --release
```
