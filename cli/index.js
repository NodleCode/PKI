#!/usr/bin/env node

require('yargs')
	.usage('Usage: $0 [--seed <seed>] <command> [options]')
	.command('new', 'Generate new signing keys that can be registered by an authority')
	.command(
		'inspect <signingPublicKey>',
		'Display the status of a slot',
		(b) => b.positional('signingPublicKey', {
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
		'book <signingPublicKey>',
		'Book a slot and link it to a given signing key',
		(b) => b.positional('signingPublicKey', {
			describe: 'the to-be-registered on-chain public signing key',
			type: 'string'
		}),
		console.log,
	)
	.command(
		'renew <signingPublicKey>',
		'Renew a given slot',
		(b) => b.positional('signingPublicKey', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}),
		console.log,
	)
	.command(
		'revoke <signingPublicKey>',
		'Revoke a slot and its associated certificates all together',
		(b) => b.positional('signingPublicKey', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}),
		console.log,
	)
	.command(
		'revoke_cert <signingPublicKey> <devicePublicKey>',
		'Revoke a certificate',
		(b) => b.positional('signingPublicKey', {
			describe: 'the registered on-chain public signing key',
			type: 'string'
		}).positional('devicePublicKey', {
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
