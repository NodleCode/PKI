#!/usr/bin/env node

//const { Certificate, Runtime } = require('pki');
const Keystore = require('./keystore');
const argv = require('yargs')
    .usage('Usage: $0 [--keystore <keystore_path>]')
    .describe('seed', 'Specify a seed used to sign transactions')
    .help()
    .epilog('copyright Nodle 2020')
    .argv;

const main = async () => {
    if (argv.keystore === undefined) {
        argv.keystore = '~/.nodle_pki_keystore.json';
    }

    const keystore = new Keystore(argv.keystore);
}

main().then(() => { process.exit(0) });