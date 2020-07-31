const cors = require('cors');
const express = require('express');
const handlers = require('./handlers');
const sleep = require('./sleep');

const enterOperatingMode = async (keystore, port, host) => {
    console.log('====== Operating Mode Enabled ======');

    const server = express();
    server.use(express.json());
    server.use(cors());
    server.get('/identity', handlers.identity(keystore));
    server.post('/challenge', handlers.challenge(keystore));
    server.post('/certificate', handlers.runtimeCertificate(keystore));
    server.listen(port, host);

    // We want this function to be blocking indefinitely.
    // If not the firmware will constantly try to go back to it.
    while (true) {
        await sleep(1000);
    }
}

module.exports = enterOperatingMode;