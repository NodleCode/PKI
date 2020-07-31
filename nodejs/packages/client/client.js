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
    async verify(runtime, onCertificateValid, onChallengeFailed, onVerificationFailed) {
        const details = await this.fetchDetails();

        if (!this.challengeDevice(details.address, onChallengeFailed)) {
            return false;
        }

        // Set to false if at least one certificate is invalid
        let ret = true;

        for (const cert of details.certificates) {
            const valid = await this.verifyIndividualCertificate(runtime, cert, details.address, onVerificationFailed);
            if (!valid) {
                ret = false; // Callback already called
            } else {
                onCertificateValid(cert);
            }
        }

        return ret;
    }

    // Send a challenge to the device to make it prove it is who it pretends to be
    async challengeDevice(deviceAddress, onChallengeFailed) {
        const challenge = crypto.randomBytes(100); // Size was chosen arbitrarily
        const challengeHex = u8aToHex(challenge);
        const reply = await axios.post(urljoin(this.url, PATH_CHALLENGE), {
            challenge: challengeHex,
        });
        const signature = reply.data.signature;
        if (signature === undefined) {
            onChallengeFailed('Invalid challenge response');
            return false;
        }
        const keyring = new Keyring({ type: 'ed25519' });
        const signerPair = keyring.addFromAddress(deviceAddress);
        try {
            const deviceSignedTheChallenge = signerPair.verify(challenge, signature);
            if (!deviceSignedTheChallenge) {
                onChallengeFailed('Challenge failed');
                return false;
            }
        } catch (e) {
            onChallengeFailed(`Signature verification error: ${e.toString()}`);
            return false;
        }

        return true;
    }

    // Sub routine to proceed to a device verification, it verifies an individual certificate
    // sequentially
    async verifyIndividualCertificate(runtime, cert, deviceAddress, onCertificateInvalid) {
        const certificateIsValid = await Certificate.verify(cert, runtime, onCertificateInvalid);
        if (!certificateIsValid) {
            return false; // Callback already called
        }

        // We know the certificate is valid but was it issued for this device
        const decodedCertificate = Certificate.decodeCertificate(cert);
        const certificateIsForThisDevice = decodedCertificate.payload.deviceAddress == deviceAddress;
        if (!certificateIsForThisDevice) {
            onCertificateInvalid(cert, 'Certificate address and device address mismatch');
            return false;
        }

        // We did it!
        return true;
    }
}

module.exports = FirmwareClient;