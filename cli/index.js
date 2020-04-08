#!/usr/bin/env node

const { Keyring } = require('@polkadot/api');
const { u8aToHex } = require('@polkadot/util');
const { randomAsU8a } = require('@polkadot/util-crypto');
const moment = require('moment');

const Certificate = require('./certificate');
const Runtime = require('./runtime');

require('yargs')
	.usage('Usage: $0 [--seed <seed>] <command> [options]')
	.command(
		'new',
		'Generate new signing keys that can be registered by an authority',
		(b) => b,
		(argv) => {
			const keyring = new Keyring({ type: 'ed25519' });
	        const seed = randomAsU8a(32);
    	    const newKey = keyring.addFromSeed(seed);

        	console.log(`Address ........ : ${newKey.address}`);
        	console.log(`Public key ..... : ${u8aToHex(newKey.publicKey)}`);
        	console.log(`Seed ........... : ${u8aToHex(seed)}`);
		},
	)
	.command(
		'inspect <signingAddress>',
		'Display the status of a slot',
		(b) => b.positional('signingAddress', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}),
		async (argv) => {
			const runtime = new Runtime(argv.wsRpc);
			await runtime.connect();
			const status = await runtime.slotStatus(argv.signingAddress);

			console.log(`Signer ......... : ${status.signingAddress}`);
			console.log(`Owner .......... : ${status.ownerAddress}`);
			console.log(`Validity ....... : ${status.valid}`);

			// Yargs doesn't play nice with promises, force exit
			process.exit(0);
		},
	)
	.command(
		'certify <deviceAddress>',
		'Forge a new certificate and sign it',
		(b) => b.positional('deviceAddress', {
			describe: 'the address of the device to certify',
			type: 'string'
		}).positional('expiry', {
			describe: 'specify in how much time the certificate expires',
			type: 'string',
			default: '1 month'
		}),
		(argv) => {
			const splitted = argv.expiry.split(" ");
			const amount = parseInt(splitted[0]);
			const unit = splitted[1];

			const keyring = new Keyring({ type: 'ed25519' });
			const pair = keyring.addFromUri(argv.seed);

			const certificate = new Certificate({ device: argv.deviceAddress, pair: pair, expiry: moment().add(amount, unit) });
			const encoded = certificate.signAndEncode();

			console.log(`Device ......... : ${certificate.deviceAddress}`);
			console.log(`Signer ......... : ${certificate.signerAddress}`);
			console.log(`Creation date .. : ${certificate.creationDate.format()}`);
			console.log(`Expiry date .... : ${certificate.expirationDate.format()}`);
			console.log('------------------')
			console.log(encoded);
		},
	)
	.command(
		'verify <certificate>',
		'Verify a given certificate by connecting to the chain',
		(b) => b.positional('certificate', {
			describe: 'the certificate to verify',
			type: 'string'
		}),
		async (argv) => {
			const runtime = new Runtime(argv.wsRpc);
			await runtime.connect();

			if (await Certificate.verify(argv.certificate, runtime)) {
				console.log('Certificate is valid');
			} else {
				console.log('Certificate is NOT valid');
			}

			process.exit(0);
		},
	)
	.command(
		'book <signingAddress>',
		'Book a slot and link it to a given signing key',
		(b) => b.positional('signingAddress', {
			describe: 'the to-be-registered on-chain public signing key',
			type: 'string'
		}),
		async (argv) => {
			const runtime = new Runtime(argv.wsRpc);
			await runtime.connect();
			runtime.setSigner(argv.seed);

			console.log(`Submitted transaction ${await runtime.bookSlot(argv.signingAddress)}`);

			process.exit(0);
		},
	)
	.command(
		'renew <signingAddress>',
		'Renew a given slot',
		(b) => b.positional('signingAddress', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}),
		async (argv) => {
			const runtime = new Runtime(argv.wsRpc);
			await runtime.connect();
			runtime.setSigner(argv.seed);

			console.log(`Submitted transaction ${await runtime.renewSlot(argv.signingAddress)}`);

			process.exit(0);
		},
	)
	.command(
		'revoke <signingAddress>',
		'Revoke a slot and its associated certificates all together',
		(b) => b.positional('signingAddress', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}),
		async (argv) => {
			const runtime = new Runtime(argv.wsRpc);
			await runtime.connect();
			runtime.setSigner(argv.seed);

			console.log(`Submitted transaction ${await runtime.revokeSlot(argv.signingAddress)}`);

			process.exit(0);
		},
	)
	.command(
		'revoke_cert <signingAddress> <deviceAddress>',
		'Revoke a certificate',
		(b) => b.positional('signingAddress', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}).positional('deviceAddress', {
			describe: 'the public key of the device',
			type: 'string'
		}),
		console.log,
	)
	.describe('seed', 'Specify a seed used to sign transactions')
	.describe('ws-rpc', 'Specify the node WS RPC endpoint, default to localhost')
	.help()
	.demandCommand()
	.epilog('copyright Nodle 2020')
	.argv;
