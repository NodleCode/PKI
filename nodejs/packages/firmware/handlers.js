const { Certificate } = require('pki');

const badRequest = (res, text) => {
    res.status(400).send({
        error: text,
    });
}

const validateAndStoreNewCertificate = (keystore, req, res) => {
    const certificate = req.body.certificate;
    if (certificate === undefined) {
        badRequest(res, 'missing certificate in post body');
        return;
    }

    // Before burning the certificate into the device (aka saving
    // it locally for us) we verify that it is for our own device.
    // We trust the issuer but want to make sure it is targeted at
    // us.
    // We wrap the call in a try catch as a decoding error may happen
    // with malicious entries.
    try {
        if (!Certificate.verifyCertificateWithoutIssuerChecks(certificate)) {
            badRequest(res, 'certificate is not for this device');
            return;
        }
    } catch (error) {
        badRequest(res, `an error happened while decoding your certificate: ${error.toString()}`);
        return;
    }

    keystore.saveCertificate(certificate);
    res.status(200).send({ accepted: true });

    console.log('Certificate received and saved');
}

module.exports = {
    identity: (keystore) => {
        return (req, res) => {
            let reply = {
                address: keystore.account.address,
                hasCertificate: keystore.hasCertificate(),
            };
            if (reply.hasCertificate) {
                reply.certificate = keystore.certificate;
            }

            res.send(reply);
        }
    },
    factoryCertificate: (keystore, shutdown) => {
        return (req, res) => {
            validateAndStoreNewCertificate(keystore, req, res);

            shutdown();
        };
    },
    challenge: (keystore) => {
        return (req, res) => {
            const challenge = req.body.challenge;
            if (challenge === undefined) {
                badRequest(res, 'missing challenge in post body');
                return;
            }

            res.send({
                signature: keystore.signChallenge(challenge),
            });
        }
    },
    runtimeCertificate: (keystore) => {
        return (req, res) => {
            validateAndStoreNewCertificate(keystore, req, res);
        }
    }
}