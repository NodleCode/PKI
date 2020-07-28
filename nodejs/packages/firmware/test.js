describe('Firmware', () => {
    context('Helpers', () => {
        context('validateAndStoreNewCertificate', () => {
            it('fails if certificate not in request in body')

            it('fails if invalid certificate')

            it('fails if certificate not for the current device')

            it('store certificate in keystore')
        })
    })

    context('Handlers', () => {
        context('Factory - Without certificates', () => {
            it('flag non presence of certificates')

            it('accept initial certificate')
        })

        context('Operating - With certificates', () => {
            it('dump certificates')

            it('accept new certificates')

            it('reply to challenges correctly')
        })
    })

    context('Modes', () => {
        it('start in factory if keystore is empty')

        it('switch to operating mode when keys provisioned')
    })
})