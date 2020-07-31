# PKI CLI and POC
A set of node js module to handle our CLI our POC.

# Development

## Building
```
docker build -t nodle/pki-utils .
```

## Testing
```
yarn
yarn run bootstrap
yarn test
```

# Usage

## CLI
```
$ docker run -it nodle/pki-utils cli --help
Usage: index.js [--seed <seed>] <command> [options]

Commands:
  index.js new                              Generate new signing keys that can
                                            be registered by an authority
  index.js inspect <signingAddress>         Display the status of a slot
  index.js certify <deviceAddress>          Forge a new certificate and sign it
  index.js verify <certificate>             Verify a given certificate by
                                            connecting to the chain
  index.js book <signingAddress>            Book a slot and link it to a given
                                            signing key
  index.js renew <signingAddress>           Renew a given slot
  index.js revoke <signingAddress>          Revoke a slot and its associated
                                            certificates all together
  index.js revoke_cert <signingAddress>     Revoke a certificate
  <deviceAddress>

Options:
  --version  Show version number                                       [boolean]
  --seed     Specify a seed used to sign transactions
  --ws-rpc   Specify the node WS RPC endpoint, default to localhost
  --help     Show help                                                 [boolean]

copyright Nodle 2020
```
