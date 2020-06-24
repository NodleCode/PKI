const { Keyring } = require('@polkadot/api');
const { randomAsU8a } = require('@polkadot/util-crypto');
const { hexToU8a, u8aToHex } = require('@polkadot/util');
const expandHomeDir = require('expand-home-dir');
const fs = require('fs');

class Keystore {
    seed = null;
    keyring = null;
    account = null;

    constructor(path) {
        const maybeExpanded = expandHomeDir(path);
        if (fs.existsSync(maybeExpanded)) {
            this.seed = this.loadSeed(maybeExpanded);
        } else {
            this.seed = this.generateAndSave(maybeExpanded);
        }

        // Load seed
        this.keyring = new Keyring({ type: 'ed25519' });
        this.account = this.keyring.addFromSeed(this.seed);
    }

    loadSeed(path) {
        const rawdata = fs.readFileSync(path);
        const parsed = JSON.parse(rawdata);

        return hexToU8a(parsed.seed);
    }

    generateAndSave(path) {
        const seed = randomAsU8a(32);

        const keystore = {
            seed: u8aToHex(seed),
        };
        const data = JSON.stringify(keystore);
        fs.writeFileSync(path, data);

        return seed;
    }
}

module.exports = Keystore;