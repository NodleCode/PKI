![web3_badge](https://user-images.githubusercontent.com/10683430/77575916-73c7ab00-6e91-11ea-8797-e24ff7c7c85c.png)

# Public Key Infrastructure
A set of modules to manage a decentralized and distributed PKI. A Token Curated Registry
is used to manage in a decentralized way signing authorities that can issue "certificates".

**We have migrated and updated the pallets present in this repository to the [main nodle chain repo](https://github.com/NodleCode/chain). This solution is becoming part of Nodle's Identity Stack.**

# Development

## Building
```
cargo build -p pki-node
```

## Testing
```
cargo test --all
```

# Docker

## Building
```
docker build -t nodle/pki-node .
```

## Running
```
$ docker run -p 9944:9944 nodle/pki-node --dev
2020-04-08 20:07:22 Running in --dev mode, RPC CORS has been disabled.
2020-04-08 20:07:22 PKI Sample Node
2020-04-08 20:07:22   version 2.0.0-09bfe46-x86_64-linux-gnu
2020-04-08 20:07:22   by Eliott Teissonniere <git.eliott@teissonniere.org>, 2019-2020
2020-04-08 20:07:22 Chain specification: Development
2020-04-08 20:07:22 Node name: truthful-sister-8980
2020-04-08 20:07:22 Roles: AUTHORITY
2020-04-08 20:07:22 Initializing Genesis block/state (state: 0x798b…e781, header-hash: 0x6d15…ec1a)
2020-04-08 20:07:22 Loading GRANDPA authority set from genesis on what appears to be first startup.
2020-04-08 20:07:22 Loaded block-time = 6000 milliseconds from genesis on first-launch
2020-04-08 20:07:22 Highest known block at #0
2020-04-08 20:07:22 Using default protocol ID "sup" because none is configured in the chain specs
2020-04-08 20:07:22 Local node identity is: QmUusiW3isU5XbbntCSmrDVrVzvW2bW6WcLARf3mEatkkz
2020-04-08 20:07:22 Prometheus server started at 127.0.0.1:9615
2020-04-08 20:07:24 Starting consensus session on top of parent 0x6d1529de9b3b87ecaa12c046b352f26c7bbb89181d608d3d6e714a6e4417ec1a
2020-04-08 20:07:24 Prepared block for proposing at 1 [hash: 0xe81fb3c88decde6e9ae87c4faff2a483f2ccd4530d352fe10d0446c9dd3a6fa9; parent_hash: 0x6d15…ec1a; extrinsics (1): [0x548b…c3df]]
2020-04-08 20:07:24 Pre-sealed block for proposal at 1. Hash now 0xecbe65159b501bf73364066db2e964460f8a7fd53bd9b1a9ef8576ba8b0ca870, previously 0xe81fb3c88decde6e9ae87c4faff2a483f2ccd4530d352fe10d0446c9dd3a6fa9.
2020-04-08 20:07:24 Imported #1 (0xecbe…a870)
```
