const { Keyring } = require('@polkadot/api');
const { randomAsU8a } = require('@polkadot/util-crypto');
const { hexToU8a, u8aToHex } = require('@polkadot/util');
const expandHomeDir = require('expand-home-dir');
const fs = require('fs');

class Keystore {
    keyring = null;
    account = null;
    certificate = undefined;
    path = null;
    seed = null;

    constructor(path) {
        this.path = expandHomeDir(path);
        if (fs.existsSync(this.path)) {
            this.loadKeystore();
        } else {
            this.generateAndSaveKeystore();
        }

        // Load seed and certificate
        this.keyring = new Keyring({ type: 'ed25519' });
        this.account = this.keyring.addFromSeed(this.seed);
    }

    loadKeystore() {
        const rawdata = fs.readFileSync(this.path);
        const parsed = JSON.parse(rawdata);

        this.seed = hexToU8a(parsed.seed);
        this.certificate = parsed.certificate;
    }

    generateAndSaveKeystore(path) {
        const seed = randomAsU8a(32);
        this.seed = seed;

        this.saveKeystore();
    }

    saveKeystore() {
        const data = JSON.stringify({
            seed: u8aToHex(this.seed),
            certificate: this.certificate,
        });
        fs.writeFileSync(this.path, data);
    }

    hasCertificate() {
        return this.certificate !== undefined;
    }

    saveCertificate(certificate) {
        this.certificate = certificate;
        this.saveKeystore();
    }

    signChallenge(challenge) {
        const decodedChallenge = hexToU8a(challenge);
        return u8aToHex(this.account.sign(decodedChallenge));
    }
}

module.exports = Keystore;