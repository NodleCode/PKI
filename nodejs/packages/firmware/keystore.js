const { Certificate, Runtime } = require('pki');

const { Keyring } = require('@polkadot/api');
const { randomAsU8a } = require('@polkadot/util-crypto');
const { hexToU8a, u8aToHex } = require('@polkadot/util');
const expandHomeDir = require('expand-home-dir');
const fs = require('fs');

class Keystore {
    seed = null;
    keyring = null;
    account = null;
    certificate = undefined;

    constructor(path) {
        const maybeExpanded = expandHomeDir(path);
        let keystore = null;
        if (fs.existsSync(maybeExpanded)) {
            keystore = this.loadKeystore(maybeExpanded);
        } else {
            keystore = this.generateAndSaveKeystore(maybeExpanded);
        }

        // Load seed and certificate
        this.keyring = new Keyring({ type: 'ed25519' });
        this.account = this.keyring.addFromSeed(keystore.seed);
        this.certificate = keystore.certificate;
    }

    loadKeystore(path) {
        const rawdata = fs.readFileSync(path);
        const parsed = JSON.parse(rawdata);

        return {
            seed: hexToU8a(parsed.seed),
            certificate: parsed.certificate,
        };
    }

    generateAndSaveKeystore(path) {
        const seed = randomAsU8a(32);

        const keystore = {
            seed: u8aToHex(seed),
        };
        const data = JSON.stringify(keystore);
        fs.writeFileSync(path, data);

        return {
            seed: seed,
            certificate: undefined,
        };
    }

    hasCertificate() {
        return this.certificate !== undefined;
    }
}

module.exports = Keystore;