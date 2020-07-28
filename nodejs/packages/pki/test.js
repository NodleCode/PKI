const { Certificate } = require('./');
const errors = require('./errors');
const { Keyring } = require('@polkadot/api');
const moment = require('moment');
const { expect } = require('chai');

describe('Certificate', () => {
    const keyring = new Keyring({ type: 'ed25519' });
    const pair = keyring.addFromUri('0xa35be3bd72de762ce02b113bbeef4c553d3ded70f014e364ff38ebeb1f3130c1');

    const paramsCert = {
        device: '5GdqmKwPke1CmjcAfTwe5dJbHqFUsYFCypRqsQBNNRDzghJJ',
        pair: pair,
        expiry: moment().add(1, 'month')
    };
    const testCert = new Certificate(paramsCert);
    const encodedCert = testCert.signAndEncode();

    context('Meta', () => {
        it('version should be 0.1', () => {
            expect(testCert.version).to.be.equal('0.1');
            expect(Certificate.decodeCertificate(encodedCert).version).to.be.equal('0.1');
        })
    })

    context('Encode / Decode', () => {
        it('encoding and decoding return the expected values', () => {
            const expected = testCert.sign();
            const decoded = Certificate.decodeCertificate(encodedCert);

            expect(decoded).to.eql(expected);
        })
    })

    context('Verify', () => {
        context('Offline', () => {
            it('return true if certificate is valid', () => {
                expect(
                    Certificate.verifyCertificateWithoutIssuerChecks(encodedCert, console.log)
                ).to.be.true;
            })

            it('fail if unsupported version', () => {
                const badCert = new Certificate(paramsCert);
                badCert.version = 'Bad';

                const encodedBadCert = badCert.signAndEncode();

                let calledBack = false;

                expect(
                    Certificate.verifyCertificateWithoutIssuerChecks(encodedBadCert, (c, r) => {
                        calledBack = true;
                        expect(r).to.be.equal(errors.errUnsupportedVersion);
                    })
                ).to.be.false;

                expect(calledBack).to.be.true;
            })

            it('fail if expired', () => {
                const badCert = new Certificate(paramsCert);
                badCert.expirationDate = moment().subtract(1, 'day');

                const encodedBadCert = badCert.signAndEncode();

                expect(
                    Certificate.verifyCertificateWithoutIssuerChecks(encodedBadCert, (c, r) => {
                        expect(r).to.be.equal(errors.errExpired);
                    })
                ).to.be.false;
            })

            it('fail if bad hash', () => {
                const decoded = Certificate.decodeCertificate(encodedCert);
                decoded.hash = '0x00';

                const encodedBadCert = Buffer.from(JSON.stringify(decoded)).toString('base64');

                expect(
                    Certificate.verifyCertificateWithoutIssuerChecks(encodedBadCert, (c, r) => {
                        expect(r).to.be.equal(errors.errHashMismatch);
                    })
                ).to.be.false;
            })

            it('fail if bad signature', () => {
                const decoded = Certificate.decodeCertificate(encodedCert);
                decoded.signature = '0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000';

                const encodedBadCert = Buffer.from(JSON.stringify(decoded)).toString('base64');

                expect(
                    Certificate.verifyCertificateWithoutIssuerChecks(encodedBadCert, (c, r) => {
                        expect(r).to.be.equal(errors.errBadSignature);
                    })
                ).to.be.false;
            })
        })

        context('Online', () => {
            it('return true if confirmed by the chain', async () => {
                const mockRuntime = {
                    rootAndChildValid: async (s, d) => {
                        return true;
                    }
                };

                expect(
                    await Certificate.verify(encodedCert, mockRuntime, console.log)
                ).to.be.true;
            })

            it('fail if chain says it is revoked or invalid', async () => {
                const mockRuntime = {
                    rootAndChildValid: async (s, d) => {
                        return false;
                    }
                };

                expect(
                    await Certificate.verify(encodedCert, mockRuntime, (c, r) => {
                        expect(r).to.be.equal(errors.errChainSaysInvalid);
                    })
                ).to.be.false;
            })
        })
    })
})