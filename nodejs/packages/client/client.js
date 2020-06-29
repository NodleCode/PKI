const axios = require('axios');
const urljoin = require('url-join');

const { Certificate } = require('pki');

const PATH_IDENTITY = '/identity';
const PATH_BURN = '/factory/certificate';

class FirmwareClient {
    url = '';

    constructor(url) {
        this.url = url;
    }

    async burn(pair, expiry) {
        const identity = await axios.get(urljoin(this.url, PATH_IDENTITY));
        const deviceAddress = identity.data.address;

        if (identity.data.hasCertificate) {
            throw new Error('Firmware no longer in factory mode');
        }

        const certificate = new Certificate({ device: deviceAddress, pair: pair, expiry: expiry });
        const encoded = certificate.signAndEncode();

        await axios.post(urljoin(this.url, PATH_BURN), {
            certificate: encoded,
        });
    }
}

module.exports = FirmwareClient;