const axios = require('axios');
const crypto = require('crypto');
const urljoin = require('url-join');
const { Keyring } = require('@polkadot/api');
const { u8aToHex } = require('@polkadot/util');

const { Certificate } = require('pki');

const PATH_IDENTITY = '/identity';
const PATH_BURN = '/factory/certificate';
const PATH_CHALLENGE = '/challenge';

class FirmwareClient {
    url = '';

    constructor(url) {
        this.url = url;
    }

    // This call can be used to 'burn' a certificate on a device running our 'firmware.
    // It will connect to it, get its public key and issue a certificate.
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

    // This call can be used to connect to a device and verify its certificate and possession
    // of its cryptographic materials by issuing it a challenge.
    async verify(runtime) {
        const identity = await axios.get(urljoin(this.url, PATH_IDENTITY));
        const certificateIsValid = Certificate.verify(identity.data.certificate, runtime);
        if (!certificateIsValid) {
            throw new Error('the certificate is not valid');
        }

        // We know the certificate is valid but was it issued for this device
        const decodedCertificate = Certificate.decodeCertificate(identity.data.certificate);
        const certificateIsForThisDevice = decodedCertificate.payload.deviceAddress == identity.data.address;
        if (!certificateIsForThisDevice) {
            throw new Error('the certificate was not issued for this device');
        }

        // The certificate is valid and for this device, but is it who it pretends to be?
        const challenge = crypto.randomBytes(100); // Size was chosen arbitrarily
        const challengeHex = u8aToHex(challenge);
        const reply = await axios.post(urljoin(this.url, PATH_CHALLENGE), {
            challenge: challengeHex,
        });
        const signature = reply.data.signature;
        if (signature === undefined) {
            throw new Error('no signature present in challenge reply');
        }
        const keyring = new Keyring({ type: 'ed25519' });
        const signerPair = keyring.addFromAddress(identity.data.address);
        const deviceSignedTheChallenge = signerPair.verify(challenge, signature);
        if (!deviceSignedTheChallenge) {
            throw new Error('the device was not able to prove ownership of its keypair');
        }

        // We did it!
        return true;
    }
}

module.exports = FirmwareClient;