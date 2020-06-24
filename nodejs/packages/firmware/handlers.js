const { Certificate } = require('pki');

const badRequest = (res, text) => {
    res.status(400).send({
        error: text,
    });
}

module.exports = {
    identity: (keystore) => {
        return (req, res) => {
            res.send({
                address: keystore.account.address,
                hasCertificate: keystore.hasCertificate(),
            });
        }
    },
    factoryCertificate: (keystore, shutdown) => {
        return (req, res) => {
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
            res.status(202).send({ accepted: true });

            shutdown();
        };
    }
}