const axios = require('axios');
const crypto = require('crypto');
const urljoin = require('url-join');
const { Keyring } = require('@polkadot/api');
const { u8aToHex } = require('@polkadot/util');

const { Certificate } = require('pki');

const PATH_IDENTITY = '/identity';
const PATH_BURN = '/certificate';
const PATH_CHALLENGE = '/challenge';

class FirmwareClient {
    constructor(url) {
        this.url = url;
    }

    // Use this call to get basic details
    async fetchDetails() {
        const identity = await axios.get(urljoin(this.url, PATH_IDENTITY));

        return {
            address: identity.data.address,
            hasCertificate: identity.data.hasCertificate,
            certificates: identity.data.certificates,
        };
    }

    // This call can be used to 'burn' a certificate on a device running our 'firmware.
    // It will connect to it, get its public key and issue a certificate.
    async burn(pair, expiry) {
        const details = await this.fetchDetails();
        const deviceAddress = details.address;

        const certificate = new Certificate({ device: deviceAddress, pair: pair, expiry: expiry });
        const encoded = certificate.signAndEncode();

        await axios.post(urljoin(this.url, PATH_BURN), {
            certificate: encoded,
        });
    }

    // This call can be used to connect to a device and verify its certificates and possession
    // of its cryptographic materials by issuing it a challenge.
    async verify(runtime) {
        const details = await this.fetchDetails();

        for (const cert of details.certificates) {
            if (!await this.verifyIndividualCertificate(runtime, cert, details.address)) {
                return false;
            }
        }

        return true;
    }

    // Sub routine to proceed to a device verification, it verifies an individual certificate
    // sequentially
    async verifyIndividualCertificate(runtime, cert, deviceAddress) {
        const certificateIsValid = Certificate.verify(cert, runtime);
        if (!certificateIsValid) {
            throw new Error('the certificate is not valid');
        }

        // We know the certificate is valid but was it issued for this device
        const decodedCertificate = Certificate.decodeCertificate(cert);
        const certificateIsForThisDevice = decodedCertificate.payload.deviceAddress == deviceAddress;
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
        const signerPair = keyring.addFromAddress(deviceAddress);
        const deviceSignedTheChallenge = signerPair.verify(challenge, signature);
        if (!deviceSignedTheChallenge) {
            throw new Error('the device was not able to prove ownership of its keypair');
        }

        // We did it!
        return true;
    }

    // This call can be used to connect to a given device and issue it a new certificate,
    // for instance, this could be useful when renewing a certificate, or maybe 
}

module.exports = FirmwareClient;