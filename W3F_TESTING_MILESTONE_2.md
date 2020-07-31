# W3F Testing Instructions for Milestone 2

## Building the docker images

> The docker images are also available in the [Github Package Registry](https://github.com/NodleCode/PKI/packages). You will want to refer to the [github documentation](https://help.github.com/en/packages/using-github-packages-with-your-projects-ecosystem/configuring-docker-for-use-with-github-packages) as to how to pull them on your local machine.

1. Build the `pki-node` image: `docker build -t nodle/pki-node .`
2. Build the `pki-utils` image: `cd nodejs && docker build -t nodle/pki-utils . && cd ..`


## Start the node

```
docker run --name pki-node -p 9944:9944 -d nodle/pki-node --dev --ws-external --rpc-cors all
```


### Cleanup

When you are done just run the following:
```
docker stop pki-node
docker rm pki-node
```


## Configure your Raspberry Pi

> The following instructions should work on any Raspberry Pi, we have been using a Pi Zero W for testing. It is needed to for the Raspberry Pi to be connected to the Internet.
> We assume that the Raspberry Pi is already configured with an existing system and is already connected with an SSH server available.

1. Log into your Raspberry Pi.
2. Update it if needed: `sudo apt update && sudo apt upgrade --yes`.
3. Install docker: `curl -fsSL https://get.docker.com | sh`. In the pizero w it might be needed to go over some system fixes, [this article](https://markmcgookin.com/2019/08/04/how-to-install-docker-on-a-raspberry-pi-zero-w-running-raspbian-buster/) was pretty useful.
4. Add the non root user: `sudo usermod -aG docker pi`. You may need to log out and back in.
5. Get our repo locally: `sudo apt install --yes git && git clone https://github.com/NodleCode/pki`.
6. Build the `pki-utils` container locally: `cd pki/nodejs && docker build -t nodle/pki-utils -f ./Dockerfile.rpi .`. This is probably going to be pretty slow depending on the Raspberry Pi model you are using.
7. Start the "firmware" and make sure it is restarted as needed: `docker run --restart always -p 8080:8080 -v `pwd`/keystore:/keystore --name firmware -d nodle/pki-utils firmware --keystore /keystore/keys.json --host 0.0.0.0`.


## Starting the POC Web App

The easiest way to access the webapp is to use the development server, you can use the following commands to start it:
```
cd nodejs
yarn
yarn webapp
```

This will open a browser straight to the webapp.


## Certifying the device

Let's setup a few more things first:
1. We are going to set an alias to make our life easier: `alias pki-cli='docker run --link pki-node -it nodle/pki-utils cli --ws-rpc ws://pki-node:9944'`.
2. Open the [the polkadot js UI](https://polkadot.js.org/apps), connect to the local node. You may need the types in `./types.json`.


### Register Alice as a Certificate Authority

Navigate to the `Extrinsics` tab on Polkadot JS and submit from `Alice` the following call: `tcr.apply(0x00, 1 Unit)`. Monitor the `Events`, after 100 blocks passed the application will be accepted (this is a runtime constant, can be changed by any integrators), an event will be triggered.


### Generate the certificate keys

Use the command `subkey --ed25519 generate` to create the root certificate keys that you will need later. We also export the seed and address as environment variables, for this example we have been using the following:
```bash
export ROOT_SEED=0x8d48cb7d36ba0783f341f9c921c8738921fc7537a0ee021a2dcce2664326fc2f
export ROOT=5EzPNjJu4DLapPcn3cz6RuSwKbD25kKHqyhCjUYZn132co4J
```

Now that you have some keys and that Alice is a Certificate Authority we can proceed to the creation of the on-chain root certificate:
```
$ pki-cli --seed //Alice book $ROOT
Unknown types found, no types for Application
Submitted transaction 0xe263ff9b32d8139c56a6385acec40402b3d0e217c9513871d605d7f29f4aa493
```


### Burn a certificate inside the device

Congrats! We can now use the root certificate to certify our device. We have added a CLI command to make it easy:
```
$ pki-cli --seed $ROOT_SEED iot_burn http://elchapo.local:8080
yarn run v1.22.4
$ node ./packages/cli/index.js --ws-rpc ws://pki-node:9944 --seed 0x8d48cb7d36ba0783f341f9c921c8738921fc7537a0ee021a2dcce2664326fc2f iot_burn http://elchapo.local:8080
Done in 1.58s.
```

You will likely have to change the `--url` parameter to the address of your Raspberry Pi (try `raspberry.local`). In our case we chose El Chapo as it is going to turn bad very soon, but more on this later.


### Verify the device's certificate

There are two ways to verify the device's certificate, let's try with the cli:
```
$ pki-cli iot_verify http://elchapo.local:8080
yarn run v1.22.4
$ node ./packages/cli/index.js --ws-rpc ws://127.0.0.1:9944 iot_verify http://elchapo.local:8080
Unknown types found, no types for Application
VALID: eyJ2ZXJzaW9uIjoiMC4xIiwicGF5bG9hZCI6eyJkZXZpY2VBZGRyZXNzIjoiNUd4aUVmaWpDYXh1RXJIM1g0Qmh5dnZZTk5pNGN2QnFHWnY2YVVWMks2QzE4ZnVRIiwic2lnbmVyQWRkcmVzcyI6IjVFelBOakp1NERMYXBQY24zY3o2UnVTd0tiRDI1a0tIcXloQ2pVWVpuMTMyY280SiIsImNyZWF0aW9uRGF0ZSI6MTU5NTg2OTg4OCwiZXhwaXJhdGlvbkRhdGUiOjE1OTg1NDgyODh9LCJoYXNoIjoiMHgwOGMyYmJlMDAxNGVkNmZkODkwYzg1MGZhMDI0NDkxYjdhNjgyYjdlYmU2ZTc3MzYzMDRiOTM5MTA1MTNjODRjYjliN2Y1MjUzZTI1M2UyM2ZiZTJjMjQwYjg1ZWE1ZTg3NjhkYzZkYjgxOWI1ZGIzNTkyNGJlYzJjNmUxMjBiNyIsInNpZ25hdHVyZSI6IjB4OGNiZWZjZDg5YjNmNDZmMGNmODk1MmJiMDUxZWVkMjU0MjIzZjRhZTFhZDdjYjNjY2Y3N2E0Zjg3YjgxOWY0MGZjNDgyODk1ZGY3ZjIzOGQ5OGE5ZDcxMmI2MTM1NzJmY2I1YjQwYTQwY2FiMTQ5MTU5OTZiMzJhYzFmMmMxMGIifQ==
Device has at least one invalid certificate
✨  Done in 1.11s.
```

But this milestone also included a web POC app, navigate to it and enter the raspberry pi url, click `Access` then `Verify`. You should see a success message!


### Bonus: add more certificates to the device

You can use the `iot_burn` command to push new certificates to the device. When verifying the device it will loop through all the current certificates of the device.


### Turning the device rogue

The goal of using certificates is to know if a device is genuine or not. Let's simulate this scenario by "compromising" the Raspberry Pi:
1. Reconnect to the raspberry pi.
2. Navigate to the `keystore` folder.
3. Edit the `keys.json` and replace one character from either the seed or certificate. This simulate a scenario where the certificate doesn't match the device key.
4. Reboot the pi: `sudo reboot`.
5. Wait a minute or two.


### Verify the rogue device's certificate

We can now process to the verification again, the `iot_verify` will have an output similar to the following (for this example we revoked the root certificate instead so you may have a slightly different error displayed):
```
pki-cli iot_verify http://elchapo.local:8080
yarn run v1.22.4
$ node ./packages/cli/index.js --ws-rpc ws://127.0.0.1:9944 iot_verify http://localhost:8080
Unknown types found, no types for Application
INVALID (Root / Child does not exist or was revoked): eyJ2ZXJzaW9uIjoiMC4xIiwicGF5bG9hZCI6eyJkZXZpY2VBZGRyZXNzIjoiNUd4aUVmaWpDYXh1RXJIM1g0Qmh5dnZZTk5pNGN2QnFHWnY2YVVWMks2QzE4ZnVRIiwic2lnbmVyQWRkcmVzcyI6IjVFelBOakp1NERMYXBQY24zY3o2UnVTd0tiRDI1a0tIcXloQ2pVWVpuMTMyY280SiIsImNyZWF0aW9uRGF0ZSI6MTU5NTg2OTg4OCwiZXhwaXJhdGlvbkRhdGUiOjE1OTg1NDgyODh9LCJoYXNoIjoiMHgwOGMyYmJlMDAxNGVkNmZkODkwYzg1MGZhMDI0NDkxYjdhNjgyYjdlYmU2ZTc3MzYzMDRiOTM5MTA1MTNjODRjYjliN2Y1MjUzZTI1M2UyM2ZiZTJjMjQwYjg1ZWE1ZTg3NjhkYzZkYjgxOWI1ZGIzNTkyNGJlYzJjNmUxMjBiNyIsInNpZ25hdHVyZSI6IjB4OGNiZWZjZDg5YjNmNDZmMGNmODk1MmJiMDUxZWVkMjU0MjIzZjRhZTFhZDdjYjNjY2Y3N2E0Zjg3YjgxOWY0MGZjNDgyODk1ZGY3ZjIzOGQ5OGE5ZDcxMmI2MTM1NzJmY2I1YjQwYTQwY2FiMTQ5MTU5OTZiMzJhYzFmMmMxMGIifQ==
Device has at least one invalid certificate
✨  Done in 0.94s.
```