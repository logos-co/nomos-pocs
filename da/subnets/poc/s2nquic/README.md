# Certificate

To genereate a new key and certificate for remote testing:

```bash
openssl ecparam -name prime256v1 -genkey -noout -out key.pem
openssl req -new -key key.pem -out cert.csr -config san.cnf
openssl x509 -req -in cert.csr -signkey key.pem -out cert.pem -days 365 -extensions req_ext -extfile san.cnf
```

# Cross compilation

To crosscompile to x86 linux target use `x86_64-unknown-linux-musl`, gcc-10 is required for `aws-lc-sys` and as of 2024-07-18, `cross` doens't have this version when using docker.

```bash
cross build --target x86_64-unknown-linux-musl --release
```
