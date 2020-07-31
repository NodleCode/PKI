const express = require('express');
const { expect } = require('chai');
const handlers = require('firmware/handlers');
const { FirmwareClient } = require('./');
const Keystore = require('firmware/keystore');
const { Keyring } = require('@polkadot/api');
const moment = require('moment');

const createKeystore = () => {
    const randomKeystoreName = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    const keystorePath = `/tmp/${randomKeystoreName}.json`;
    const keystore = new Keystore(keystorePath);

    return keystore;
}

const buildMockDevice = () => {
    const keystore = createKeystore();

    // For testing, we build a csustom server with all the
    // handlers we want to test and thus can abstract away
    // from device mode related things.
    const server = express();
    server.use(express.json());
    server.get('/identity', handlers.identity(keystore));
    server.post('/challenge', handlers.challenge(keystore));
    server.post('/certificate', handlers.runtimeCertificate(keystore));

    // Make sure this port is free!
    const shutdown = server.listen(5769, 'localhost');

    const client = new FirmwareClient('http://localhost:5769');

    return {
        server: server,
        shutdown: shutdown,
        keystore: keystore,
        client: client
    };
}

const evilVariants = {
    BAD_RESPONSE: 1,
    BAD_SIGNATURE_FORMAT: 2,
    BAD_SIGNATURE: 3
}

const buildMockEvilDevice = (variant) => {
    const keystore = createKeystore();

    const server = express();
    server.use(express.json());
    server.get('/identity', handlers.identity(keystore));
    server.post('/challenge', (req, res) => {
        if (variant == evilVariants.BAD_RESPONSE) {
            res.send({ fake: true });
        } else if (variant == evilVariants.BAD_SIGNATURE_FORMAT) {
            res.send({ signature: '0xdeadbeef' });
        } else if (variant == evilVariants.BAD_SIGNATURE) {
            res.send({ signature: '0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000' });
        } else {
            throw new Error('unknown variant');
        }
    });

    const shutdown = server.listen(5768, 'localhost');
    const client = new FirmwareClient('http://localhost:5768');

    return {
        server: server,
        shutdown: shutdown,
        keystore: keystore,
        client: client
    };
}

const createSigningPair = () => {
    const keyring = new Keyring({ type: 'ed25519' });
    const pair = keyring.addFromUri('0x8bfc1b8f605890627bfcef37ca884258e64d9fc7b91a5aefc4851c7103468b46');

    return pair;
}

const buildMockRuntime = (ret) => {
    return {
        rootAndChildValid: async (unusedA, unusedB) => {
            return ret;
        }
    }
}

describe('Client to Firmware', () => {
    const { shutdown, keystore, client } = buildMockDevice();
    const pair = createSigningPair();

    after(() => {
        shutdown.close();
    })

    context('Metadata', () => {
        it('fetch and return the correct identity informations', async () => {
            const details = await client.fetchDetails();

            // No certificate was burnt
            expect(details).to.be.eql({
                address: keystore.account.address,
                hasCertificate: false,
                certificates: undefined
            });
        })
    })

    context('Burning new certificates', () => {
        it('send a valid certificate to the correct endpoint', async () => {
            await client.burn(pair, moment().add(1, 'month'));

            const details = await client.fetchDetails();

            expect(details.hasCertificate).to.be.true;
            expect(details.certificates.length).to.be.equal(1);
        })
    })

    context('Verifications', () => {
        context('Genuine Device', () => {
            it('challenge works', async () => {
                const details = await client.fetchDetails();
                const challengeSuccess = await client.challengeDevice(details.address, console.log);
                expect(challengeSuccess).to.be.true;
            })

            it('complete verification works', async () => {
                const success = await client.verify(buildMockRuntime(true), console.log, console.log, console.log);
                expect(success).to.be.true;
            })
        })

        context('Rogue Device', () => {
            context('Challenges', () => {
                it('fail if bad response', async () => {
                    const { shutdown, client } = buildMockEvilDevice(evilVariants.BAD_RESPONSE);

                    const success = await client.challengeDevice('5FJBxp47Ss3aZVX6L5j27mHvXpZXBsVSXpucRgsmiiULztAt', console.log);
                    expect(success).to.be.false;

                    shutdown.close();
                })

                it('fail if bad signature format', async () => {
                    const { shutdown, client } = buildMockEvilDevice(evilVariants.BAD_SIGNATURE_FORMAT);

                    const success = await client.challengeDevice('5FJBxp47Ss3aZVX6L5j27mHvXpZXBsVSXpucRgsmiiULztAt', console.log);
                    expect(success).to.be.false;

                    shutdown.close();
                })

                it('fail if bad signature', async () => {
                    const { shutdown, client } = buildMockEvilDevice(evilVariants.BAD_SIGNATURE);

                    const success = await client.challengeDevice('5FJBxp47Ss3aZVX6L5j27mHvXpZXBsVSXpucRgsmiiULztAt', console.log);
                    expect(success).to.be.false;

                    shutdown.close();
                })
            })

            context('Certificates', () => {
                it('fail on PKI verification failure', async () => {
                    const success = await client.verify(buildMockRuntime(false), console.log, console.log, console.log);
                    expect(success).to.be.false;
                })
            })
        })
    })
})