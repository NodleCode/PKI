const Keystore = require("./keystore")
const { Certificate } = require('pki');
const { Keyring } = require('@polkadot/api');
const { randomAsU8a } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');
const moment = require('moment');
const { expect } = require("chai");

const mockCert = (signer, device) => {
    const keyring = new Keyring({ type: 'ed25519' });
    const pair = keyring.addFromUri(signer);

    const paramsCert = {
        device: device,
        pair: pair,
        expiry: moment().add(1, 'month')
    };
    const cert = new Certificate(paramsCert);
    return cert.signAndEncode();
}

describe('Firmware', () => {
    context('Helpers', () => {
        context('validateAndStoreNewCertificate', () => {
            it('fails if certificate not in request in body')

            it('fails if invalid certificate')

            it('fails if certificate not for the current device')

            it('store certificate in keystore')
        })
    })

    context('Handlers', () => {
        context('Factory - Without certificates', () => {
            it('flag non presence of certificates')

            it('accept initial certificate')
        })

        context('Operating - With certificates', () => {
            it('dump certificates')

            it('accept new certificates')

            it('reply to challenges correctly')
        })
    })

    context('Modes', () => {
        it('start in factory if keystore is empty')

        it('switch to operating mode when keys provisioned')
    })
})

describe('Keystore', () => {
    const randomKeystoreName = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    const keystorePath = `/tmp/${randomKeystoreName}.json`;

    const keystore = new Keystore(keystorePath);

    it('generates an empty on disk file', () => {
        expect(require('fs').existsSync(keystorePath)).to.be.true;
    })

    context('Certificates', () => {
        it('starts empty', () => {
            expect(keystore.hasCertificate()).to.be.false;
        })

        it('load and save correctly with one certificate', () => {
            const cert = mockCert('0xa35be3bd72de762ce02b113bbeef4c553d3ded70f014e364ff38ebeb1f3130c1', keystore.account.address);
            keystore.saveCertificate(cert);

            expect(keystore.hasCertificate()).to.be.true;

            const loaded = new Keystore(keystorePath);
            expect(loaded.hasCertificate()).to.be.true;
            expect(loaded.certificates.length).to.be.equal(1);
            expect(loaded.certificates[0]).to.be.equal(cert);
        })

        it('load and save correctly with many certificates', () => {
            const cert = mockCert('0xbd1992343b5ea318149a0646aef6162306f45ec3346abfdea6d5ac22788c1520', keystore.account.address);
            keystore.saveCertificate(cert);

            expect(keystore.hasCertificate()).to.be.true;

            const loaded = new Keystore(keystorePath);
            expect(loaded.hasCertificate()).to.be.true;
            expect(loaded.certificates.length).to.be.equal(2);
            expect(loaded.certificates).to.contain(cert);
        })
    })

    context('Challenges', () => {
        it('succesfully solve challenges', () => {
            const challenge = randomAsU8a(32);
            const signed = keystore.signChallenge(u8aToHex(challenge));

            expect(keystore.account.verify(challenge, signed)).to.be.true;
        })
    })
})