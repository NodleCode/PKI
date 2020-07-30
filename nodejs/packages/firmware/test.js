const Keystore = require("./keystore");
const errors = require('./errors');
const { Certificate } = require('pki');
const { Keyring } = require('@polkadot/api');
const { randomAsU8a } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');
const moment = require('moment');
const chai = require('chai');
const { expect } = require('chai');
const chaiHttp = require('chai-http');
const express = require('express');
const handlers = require('./handlers');

chai.use(chaiHttp);

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

const testKeystore = () => {
    const randomKeystoreName = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    const keystorePath = `/tmp/${randomKeystoreName}.json`;
    const keystore = new Keystore(keystorePath);

    return {
        keystorePath: keystorePath,
        keystore: keystore,
    }
}

describe('Firmware', () => {
    const { keystore } = testKeystore();
    const cert = mockCert('0xccb0bc91495862000c9e1bd39ba59bb2544bd336d8eeebc662e3c1e4ad33688f', keystore.account.address);
    const otherCert = mockCert('0x2486f51d9b3febccca1a5a3e07d109a3d7a998508028ecb343da7e20c6867d90', keystore.account.address);

    // For testing, we build a csustom server with all the
    // handlers we want to test and thus can abstract away
    // from device mode related things.
    const server = express();
    server.use(express.json());
    server.get('/common/identity', handlers.identity(keystore));
    server.post('/factory/certificate', handlers.factoryCertificate(keystore, () => { }));
    server.post('/operating/challenge', handlers.challenge(keystore));
    server.post('/operating/certificate', handlers.runtimeCertificate(keystore));

    // Make sure this port is free!
    const shutdown = server.listen(5768);

    after(() => {
        // Turn off the express server
        shutdown.close();
    })

    context('Handlers', () => {
        context('Factory Mode', () => {
            it('flag non presence of certificates', async () => {
                await chai.request(server)
                    .get('/common/identity')
                    .then((res) => {
                        expect(res).to.have.status(200);
                        expect(res).to.be.json;

                        expect(res.body.address).to.be.equal(keystore.account.address);
                        expect(res.body.hasCertificate).to.be.false;
                    });
            })

            it('accept initial certificate', async () => {
                await chai.request(server)
                    .post('/factory/certificate')
                    .send({ certificate: cert })
                    .then((res) => {
                        expect(res).to.have.status(200);
                        expect(res).to.be.json;

                        expect(res.body.accepted).to.be.true;
                        expect(keystore.certificates[0]).to.be.equal(cert);
                    });
            })
        })

        context('Operating Mode', () => {
            it('dump certificates', async () => {
                await chai.request(server)
                    .get('/common/identity')
                    .then((res) => {
                        expect(res).to.have.status(200);
                        expect(res).to.be.json;

                        expect(res.body.address).to.be.equal(keystore.account.address);
                        expect(res.body.hasCertificate).to.be.true;
                        expect(res.body.certificates).to.be.eql(keystore.certificates);
                    });
            })

            it('accept new certificates', async () => {
                await chai.request(server)
                    .post('/operating/certificate')
                    .send({ certificate: otherCert })
                    .then((res) => {
                        expect(res).to.have.status(200);
                        expect(res).to.be.json;

                        expect(res.body.accepted).to.be.true;
                        expect(keystore.certificates.length).to.be.equal(2);
                        expect(keystore.certificates).to.contain(cert);
                        expect(keystore.certificates).to.contain(otherCert);
                    });
            })

            it('reply to challenges correctly', async () => {
                const challenge = randomAsU8a(32);

                await chai.request(server)
                    .post('/operating/challenge')
                    .send({ challenge: u8aToHex(challenge) })
                    .then((res) => {
                        expect(res).to.have.status(200);
                        expect(res).to.be.json;

                        expect(keystore.account.verify(challenge, res.body.signature)).to.be.true;
                    });
            })
        })

        context('Certificates Upload - Error Cases', () => {
            it('fails if certificate not in request body', async () => {
                await chai.request(server)
                    .post('/factory/certificate') // No body
                    .then((res) => {
                        expect(res).to.have.status(400);
                        expect(res).to.be.json;

                        expect(res.body.error).to.be.equal(errors.errMissingCertificate);
                    });
            })

            it('fails if invalid certificate', async () => {
                const keyring = new Keyring({ type: 'ed25519' });
                const pair = keyring.addFromUri('0x82afb1db969e2d76e23f5e4426929550657d86a2cfa3691cf8933454945cf03e');

                const paramsCert = {
                    device: keystore.account.address,
                    pair: pair,
                    expiry: moment().subtract(1, 'month') // This certificate is already expired
                };
                const expiredCert = new Certificate(paramsCert);
                const expiredEncodedCert = expiredCert.signAndEncode();

                await chai.request(server)
                    .post('/factory/certificate')
                    .send({ certificate: expiredEncodedCert })
                    .then((res) => {
                        expect(res).to.have.status(400);
                        expect(res).to.be.json;

                        expect(res.body.error).to.be.equal('Invalid certificate: Expired');
                    });
            })

            it('fails if certificate not for the current device', async () => {
                const badCert = mockCert('0x5da049a9a390b0b5a445ad7e4eb49e3403382132ff1453d31b717a19aa518096', '5HcgBqLVgS9rriLx6J1nKBko7DAS3VrDrbBbf8vBYEGzuB7j');

                await chai.request(server)
                    .post('/factory/certificate')
                    .send({ certificate: badCert })
                    .then((res) => {
                        expect(res).to.have.status(400);
                        expect(res).to.be.json;

                        expect(res.body.error).to.be.equal(errors.errNotForThisDevice);
                    });
            })
        })
    })
})

describe('Keystore', () => {
    const { keystore, keystorePath } = testKeystore();

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