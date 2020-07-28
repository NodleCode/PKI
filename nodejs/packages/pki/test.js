describe('Certificate', () => {
    context('Encode / Decode', () => {
        it('encoding and decoding return the expected values')
    })

    context('Verify', () => {
        it('return true if certificate is valid')
        it('fail if unsupported version')
        it('fail if expired')
        it('fail if bad hash')
        it('fail if bad signature')
        it('fail if chain says it is revoked or invalid')
    })
})