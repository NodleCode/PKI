const moment = require('moment');

class Certificate {
	deviceAddress = '';
	signerAddress = '';
	signerKeypair = null;
	creationDate = null;
	expirationDate = null;

	constructor(description) {
		this.deviceAddress = description.device;
		this.signerKeypair = description.pair;
		this.signerAddress = description.pair.address;
		this.creationDate = moment();
		this.expirationDate = description.expiry;
	}

	signAndEncode() {
		return ''
	}
}

module.exports = Certificate;