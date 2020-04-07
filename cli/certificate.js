const { stringToU8a, u8aToHex, u8aToU8a } = require('@polkadot/util');
const blake = require('blakejs')
const moment = require('moment');

class Certificate {
	devicePublicKey = '';
	signerAddress = '';
	signerKeypair = null;
	creationDate = null;
	expirationDate = null;

	constructor(description) {
		this.devicePublicKey = description.device;
		this.signerKeypair = description.pair;
		this.signerAddress = description.pair.address;
		this.creationDate = moment();
		this.expirationDate = description.expiry;
	}

	sign() {
		const rawMessage = {
			devicePublicKey: this.devicePublicKey,
			signerAddress: this.signerAddress,
			creationDate: this.creationDate,
			expirationDate: this.expirationDate
		};

		const u8aMessage = u8aToU8a(this.devicePublicKey)
			+ this.signerKeypair.publicKey
			+ new Uint8Array([this.creationDate.unix()])
			+ new Uint8Array([this.expirationDate.unix()]);

		const u8aHash = blake.blake2b(u8aMessage);
		const u8aSignature = this.signerKeypair.sign(u8aHash);

		return {
			version: '0.1',
			payload: rawMessage,
			hash: u8aToHex(u8aHash),
			signature: u8aToHex(u8aSignature)
		}
	}

	signAndEncode() {
		const signed = this.sign();

		return Buffer.from(JSON.stringify(signed)).toString('base64')
	}
}

module.exports = Certificate;