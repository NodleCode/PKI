const { Keyring } = require('@polkadot/api');
const { randomAsU8a } = require('@polkadot/util-crypto');
const { hexToU8a, u8aToHex } = require('@polkadot/util');
const expandHomeDir = require('expand-home-dir');
const fs = require('fs');

class Keystore {
    constructor(path) {
        this.certificates = [];
        this.path = expandHomeDir(path);
        if (fs.existsSync(this.path)) {
            this.loadKeystore();
        } else {
            this.generateAndSaveKeystore();
        }

        this.keyring = new Keyring({ type: 'ed25519' });
        this.account = this.keyring.addFromSeed(this.seed);
    }

    loadKeystore() {
        const rawdata = fs.readFileSync(this.path);
        const parsed = JSON.parse(rawdata);

        this.seed = hexToU8a(parsed.seed);
        this.certificates = parsed.certificates;
    }

    generateAndSaveKeystore() {
        const seed = randomAsU8a(32);
        this.seed = seed;

        this.saveKeystore();
    }

    saveKeystore() {
        const data = JSON.stringify({
            seed: u8aToHex(this.seed),
            certificates: this.certificates,
        });
        fs.writeFileSync(this.path, data);
    }

    hasCertificate() {
        return this.certificates.length > 0;
    }

    saveCertificate(certificate) {
        this.certificates.push(certificate);
        this.saveKeystore();
    }

    signChallenge(challenge) {
        const decodedChallenge = hexToU8a(challenge);
        return u8aToHex(this.account.sign(decodedChallenge));
    }
}

module.exports = Keystore;