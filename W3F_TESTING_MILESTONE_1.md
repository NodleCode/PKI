# W3F Testing Instructions

## Building the docker images

> The docker images are also available in the
> [Github Package Registry](https://github.com/NodleCode/PKI/packages). You
> will want to refer to the [github documentation](https://help.github.com/en/packages/using-github-packages-with-your-projects-ecosystem/configuring-docker-for-use-with-github-packages)
> as to how to pull them on your local machine.

1. Build the `pki-node` image: `docker build -t nodle/pki-node .`
2. Build the `pki-utils` image: `cd nodes && docker build -t nodle/pki-utils .`


## Start the node

```
docker run --name pki-node -p 9944:9944 -d nodle/pki-node --dev --ws-external
```


### Cleanup

When you are done just run the following:
```
docker stop pki-node
docker rm pki-node
```


## Use the TCR


## Use the CLI to interact with the Root Of Trust

The Root Of Trust is the pallet managing the certificate authorities, you can
use it to register new root authorities and then verify certificates.

We will go through a sample flow by using the cli to interact with the Root Of
Trust pallet.

To simplify commands let's define an alias:
`alias pki-cli='docker run --link pki-node -it nodle/pki-utils cli --ws-rpc ws://pki-node:9944'`.


Make sure to open https://polkadot.js.org/apps and connect to the local development node.
You may want to use the content of the `./types.json` file in the developer tab.


### Register a new root authority

A root authority key can be used to author certificates.

Navigate to `Extrinsics` tab and submit from `Alice` the following call:
`tcr.apply(0x, 1 Unit)`. Monitor the `Events`, after 100 blocks passed the application will
be accepted (this is a runtime constant, can be changed by any integrators), an event will
be triggered.


### Generate a root key

Our CLI uses `ed25519` keys to manipulate the roots for compatibility with existing Secure Elements.
```
$ subkey --ed25519 generate
Secret phrase `today slot immense twist eternal stock toy skirt trial lava awful print` is account:
  Secret seed:      0x8bc501fd0cae72d698f3d4871df0aaf00c90d86d63aeae397a74a24f51f7b987
  Public key (hex): 0x322e4a29a94c12325806b32e6b38215bca0f9b2872b230976bd0bf3e9ad8e156
  Account ID:       0x322e4a29a94c12325806b32e6b38215bca0f9b2872b230976bd0bf3e9ad8e156
  SS58 Address:     5DCW1622FHLYT5A1qc4QP8QBuSzaeRD97Sv4XCES5pvVQ2V9
```

Let's define it as a variable: `export ROOT=5DCW1622FHLYT5A1qc4QP8QBuSzaeRD97Sv4XCES5pvVQ2V9` and same
for the seed `export ROOT_SEED=0x8bc501fd0cae72d698f3d4871df0aaf00c90d86d63aeae397a74a24f51f7b987`.



### Register a root key

We can now use `//Alice` to register a signing key, to make it easy we will use Ferdy's account.
An event will be triggered.

```
$ pki-cli --seed //Alice book $ROOT
Unknown types found, no types for Application
Submitted transaction 0xe263ff9b32d8139c56a6385acec40402b3d0e217c9513871d605d7f29f4aa493
```

We can now inspect its status:
```
$ pki-cli inspect $ROOT
Unknown types found, no types for Application
Signer ......... : 5DCW1622FHLYT5A1qc4QP8QBuSzaeRD97Sv4XCES5pvVQ2V9
Owner .......... : 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
Validity ....... : true
```


### Create a certificate

Let's certify the device with the key `5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw` (Eve).
The certificate is outputted as a base64 string, it will be differed in your environment.

```
$ pki-cli --seed $ROOT_SEED certify 5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw
Device ......... : 5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw
Signer ......... : 5DCW1622FHLYT5A1qc4QP8QBuSzaeRD97Sv4XCES5pvVQ2V9
Creation date .. : 2020-04-10T18:20:22+00:00
Expiry date .... : 2020-05-10T18:20:22+00:00
------------------
eyJ2ZXJzaW9uIjoiMC4xIiwicGF5bG9hZCI6eyJkZXZpY2VBZGRyZXNzIjoiNUhHaldBZUZEZkZDV1BzakZRZFZWMk1zdnoyWHRNa3R2Z29jRVpjQ2o2OGtVTWF3Iiwic2lnbmVyQWRkcmVzcyI6IjVEQ1cxNjIyRkhMWVQ1QTFxYzRRUDhRQnVTemFlUkQ5N1N2NFhDRVM1cHZWUTJWOSIsImNyZWF0aW9uRGF0ZSI6MTU4NjU0MjgyMiwiZXhwaXJhdGlvbkRhdGUiOjE1ODkxMzQ4MjJ9LCJoYXNoIjoiMHg0MGFmYzc3MGU3NTg5NGVjNWVhMmI3MGVhN2ZmZTgwZmU5YmUwNTViYmEzMzViZjkzMWU1ODk0NjQ0ODU4NTIxYTE3M2ZiM2ZiMjcyOGQ0NzRkZjUzNGY2OTY1OTZlY2NkOGY0ZDk2ZjQ4Njg4OTI3Y2FiMmQyNmJlNzY0MmRlMCIsInNpZ25hdHVyZSI6IjB4YzhiNWE4MWRhMzcyNzFiMzY4YTM3MjA0ZDdhYWEyM2JjMDFlNDQzMzk4YTNjOTU1ZWY0ZDhmZDI1MDc4Y2MyYjkzMjQ4MzIyNTBhNmIyZDc1OTFhYjI4ZGZkYmZhNDliN2ZkMzc5ZjgxMjViMGY2MTJjMmQ3NjI1YmZjYjI3MDgifQ==
```

For easier usage let's make a variable: `export CERT=eyJ2ZXJzaW9uIjoiMC4xIiwicGF5bG9hZCI6eyJkZXZpY2VBZGRyZXNzIjoiNUhHaldBZUZEZkZDV1BzakZRZFZWMk1zdnoyWHRNa3R2Z29jRVpjQ2o2OGtVTWF3Iiwic2lnbmVyQWRkcmVzcyI6IjVEQ1cxNjIyRkhMWVQ1QTFxYzRRUDhRQnVTemFlUkQ5N1N2NFhDRVM1cHZWUTJWOSIsImNyZWF0aW9uRGF0ZSI6MTU4NjU0MjgyMiwiZXhwaXJhdGlvbkRhdGUiOjE1ODkxMzQ4MjJ9LCJoYXNoIjoiMHg0MGFmYzc3MGU3NTg5NGVjNWVhMmI3MGVhN2ZmZTgwZmU5YmUwNTViYmEzMzViZjkzMWU1ODk0NjQ0ODU4NTIxYTE3M2ZiM2ZiMjcyOGQ0NzRkZjUzNGY2OTY1OTZlY2NkOGY0ZDk2ZjQ4Njg4OTI3Y2FiMmQyNmJlNzY0MmRlMCIsInNpZ25hdHVyZSI6IjB4YzhiNWE4MWRhMzcyNzFiMzY4YTM3MjA0ZDdhYWEyM2JjMDFlNDQzMzk4YTNjOTU1ZWY0ZDhmZDI1MDc4Y2MyYjkzMjQ4MzIyNTBhNmIyZDc1OTFhYjI4ZGZkYmZhNDliN2ZkMzc5ZjgxMjViMGY2MTJjMmQ3NjI1YmZjYjI3MDgifQ==`.
You will want to use your own outputted base64 certificate here.


### Verify a certificate

When the certificate was just created it should be valid, let's verify it!

```
$ pki-cli verify $CERT
Unknown types found, no types for Application
Certificate is valid
```


### Revoke the certificate

Oops! The key from `Ferdie` has been compromised, let's revoke it. As always, an event
will be triggered.

```
$ pki-cli --seed //Alice revoke $ROOT
Unknown types found, no types for Application
Submitted transaction 0x1e1cd90e42439c3e4fc1a578a6d926b78ffa3276d35d0a163c7dde15806abe53
```

Let's verify:
```
$ pki-cli inspect $ROOT
Unknown types found, no types for Application
Signer ......... : 5DCW1622FHLYT5A1qc4QP8QBuSzaeRD97Sv4XCES5pvVQ2V9
Owner .......... : 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
Validity ....... : false
```

And now, notice how the certificate is invalidated.
```
$ pki-cli verify $CERT
Unknown types found, no types for Application
Root / Child not valid or revoked
Certificate is NOT valid
```