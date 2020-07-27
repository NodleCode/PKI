const cors = require('cors');
const express = require('express');
const handlers = require('./handlers');
const sleep = require('./sleep');

const enterFactoryMode = async (keystore, port, host) => {
    console.log('====== Factory Mode Enabled ======');
    console.log('Awaiting certificate burning instructions...');

    // Listen starts the server in the background so we have to do
    // some wizardy to return only when the server stops (aka when
    // a certificate is burned into the device).
    let shutdownCalled = false;
    let closer = null;
    const server = express();
    const shutdownTheServer = () => {
        closer.close();
        shutdownCalled = true;
    };
    server.use(express.json());
    server.use(cors());
    server.get('/identity', handlers.identity(keystore));
    server.post('/certificate', handlers.factoryCertificate(keystore, shutdownTheServer));
    closer = server.listen(port, host);

    while (!shutdownCalled) {
        await sleep(100);
    }
}

module.exports = enterFactoryMode;