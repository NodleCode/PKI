const { Keyring } = require('@polkadot/api');
const { u8aToHex, u8aToU8a } = require('@polkadot/util');
const blake = require('blakejs')
const moment = require('moment');

class Certificate {
	constructor(description) {
		this.deviceAddress = description.device;
		this.signerKeypair = description.pair;
		this.signerAddress = description.pair.address;
		this.creationDate = moment();
		this.expirationDate = description.expiry;
	}

	sign() {
		const rawMessage = {
			deviceAddress: this.deviceAddress,
			signerAddress: this.signerAddress,
			creationDate: this.creationDate.unix(),
			expirationDate: this.expirationDate.unix()
		};

		const u8aMessage = u8aToU8a(rawMessage.deviceAddress)
			+ this.signerKeypair.publicKey
			+ new Uint8Array([rawMessage.creationDate])
			+ new Uint8Array([rawMessage.expirationDate]);
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

	static decodeCertificate(encodedCertificate) {
		const buff = Buffer.from(encodedCertificate, 'base64');
		const json = buff.toString('ascii');
		return JSON.parse(json);
	}

	static verifyCertificateWithoutIssuerChecks(encodedCertificate, onCertificateInvalid) {
		const decoded = this.decodeCertificate(encodedCertificate);

		if (decoded.version !== '0.1') {
			onCertificateInvalid(encodedCertificate, 'Unsupported version');
			return false;
		}

		const expired = moment.unix(decoded.payload.expirationDate).isBefore();
		if (expired) {
			onCertificateInvalid(encodedCertificate, 'Expired');
			return false
		}

		const keyring = new Keyring({ type: 'ed25519' });
		const signerPair = keyring.addFromAddress(decoded.payload.signerAddress);

		const u8aMessage = u8aToU8a(decoded.payload.deviceAddress)
			+ signerPair.publicKey
			+ new Uint8Array([decoded.payload.creationDate])
			+ new Uint8Array([decoded.payload.expirationDate]);
		const u8aHash = blake.blake2b(u8aMessage);

		const hashMatch = decoded.hash == u8aToHex(u8aHash);
		if (!hashMatch) {
			onCertificateInvalid(encodedCertificate, 'Hash mismatch');
			return false
		}

		const signatureOk = signerPair.verify(u8aHash, u8aToU8a(decoded.signature));
		if (!signatureOk) {
			onCertificateInvalid(encodedCertificate, 'Bad signature')
			return false
		}

		return true;
	}

	static async verify(encodedCertificate, runtime, onCertificateInvalid) {
		if (!this.verifyCertificateWithoutIssuerChecks(encodedCertificate, onCertificateInvalid)) {
			return false; // Callback already called
		}

		const decoded = this.decodeCertificate(encodedCertificate);
		const chainStateOk = await runtime.rootAndChildValid(decoded.payload.signerAddress, decoded.payload.deviceAddress);
		if (chainStateOk == false) {
			onCertificateInvalid(encodedCertificate, 'Root / Child does not exist or was revoked')
			return false;
		}

		return true;
	}
}

module.exports = Certificate;