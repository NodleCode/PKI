#!/usr/bin/env node

const Keystore = require('./keystore');
const argv = require('yargs')
    .usage('Usage: $0 [--keystore <keystore_path>]')
    .describe('seed', 'Specify a seed used to sign transactions')
    .help()
    .epilog('copyright Nodle 2020')
    .argv;

const enterFactoryMode = () => {
    console.log('Factory mode');
}

const enterOperatingMode = () => {
    console.log('Operating mode');
}

const main = async () => {
    if (argv.keystore === undefined) {
        argv.keystore = '~/.nodle_pki_keystore.json';
    }

    // First we load the local devices keys, if none are present
    // we generate them. Basically we generate the keys locally
    // on the first boot.
    const keystore = new Keystore(argv.keystore);

    // If no certificates are present we enter the 'Factory Mode',
    // basically we wait for an operator to burn our certificate
    // into the device.
    // We apply the trust on first use principle, which means that
    // whoever burn the certificate first is trusted. In a real
    // world production scenario the keys would be managed through
    // a secure element and the certificate provisioned at the
    // factory in the assembly lines.
    if (!keystore.hasCertificate()) {
        enterFactoryMode();
    } else {
        enterOperatingMode();
    }
}

main().then(() => { process.exit(0) });