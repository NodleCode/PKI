#!/usr/bin/env node

const { Keyring } = require('@polkadot/api');
const { u8aToHex } = require('@polkadot/util');
const { randomAsU8a } = require('@polkadot/util-crypto');

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
		console.log,
	)
	.command(
		'certify <deviceKey>',
		'Forge a new certificate and sign it',
		(b) => b.positional('deviceKey', {
			describe: 'the public key of the device to certify',
			type: 'string'
		}),
		console.log,
	)
	.command(
		'verify <certificate>',
		'Verify a given certificate by connecting to the chain',
		(b) => b.positional('certificate', {
			describe: 'the certificate to verify',
			type: 'string'
		}),
		console.log,
	)
	.command(
		'book <signingAddress>',
		'Book a slot and link it to a given signing key',
		(b) => b.positional('signingAddress', {
			describe: 'the to-be-registered on-chain public signing key',
			type: 'string'
		}),
		console.log,
	)
	.command(
		'renew <signingAddress>',
		'Renew a given slot',
		(b) => b.positional('signingAddress', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}),
		console.log,
	)
	.command(
		'revoke <signingAddress>',
		'Revoke a slot and its associated certificates all together',
		(b) => b.positional('signingAddress', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}),
		console.log,
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
