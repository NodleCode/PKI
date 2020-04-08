const { ApiPromise, WsProvider } = require('@polkadot/api');

class Runtime {
	api = null;
	provider = null;

	constructor(wsRpcUrl) {
		if (wsRpcUrl == undefined || wsRpcUrl == null || wsRpcUrl == "") {
			wsRpcUrl = 'ws://localhost:9944';
		}

		this.provider = new WsProvider(wsRpcUrl);
	}

	async connect() {
		this.api = await ApiPromise.create({
			provider: this.provider,
			types: {
				CertificateId: "AccountId",
				Address: "AccountId",
				RootCertificate: {
					owner: "AccountId",
					key: "CertificateId",
					created: "BlockNumber",
					renewed: "BlockNumber",
					revoked: "bool",
					validity: "BlockNumber",
					child_revocations: "Vec<CertificateId>"
				}
			},
			rpc: {
				rootOfTrust: {
					isRootCertificateValid: {
						description: "Verify if a root certificate is valid",
						params: [{
							name: "cert",
							type: "CertificateId",
						}],
						type: "bool"
					}
				}
			}
		})
	}

	async slotStatus(signerAddress) {
		const slot = await this.api.query.rootOfTrust.slots(signerAddress);
		const isValid = await this.api.rpc.rootOfTrust.isRootCertificateValid(signerAddress);
		
		return {
			signingAddress: slot.key,
			ownerAddress: slot.owner,
			valid: isValid
		}
	}
}

module.exports = Runtime;