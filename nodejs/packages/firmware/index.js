#!/usr/bin/env node

const Keystore = require('./keystore');
const enterFactoryMode = require('./mode_factory');
const argv = require('yargs')
    .usage('Usage: $0 [--keystore <keystore_path>] [--port <server_listening_port>] [--host <server_host>]')
    .describe('seed', 'Specify a seed used to sign transactions')
    .help()
    .epilog('copyright Nodle 2020')
    .argv;

const enterOperatingMode = () => { }

const main = async () => {
    if (argv.keystore === undefined) {
        argv.keystore = '~/.nodle_pki_keystore.json';
    }

    if (argv.port === undefined) {
        argv.port = 8080;
    }

    if (argv.host === undefined) {
        argv.host = 'localhost';
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
    while (true) {
        if (!keystore.hasCertificate()) {
            await enterFactoryMode(keystore, argv.port, argv.host);
        } else {
            enterOperatingMode();
        }
    }
}

main().then(() => { process.exit(0) });