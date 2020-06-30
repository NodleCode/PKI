import React from 'react';
import { Button, Heading, Pane, Spinner, Text } from 'evergreen-ui';

import { FirmwareClient } from 'client';

class DeviceDetails extends React.Component {
    constructor(props) {
        super(props);
        this.state = { address: '', hasCertificate: false }
    }

    componentDidMount() {
        this.getDeviceDetails(this.props.url);
    }

    async getDeviceDetails(url) {
        const client = new FirmwareClient(url);
        const details = await client.fetchDetails();

        this.setState({ address: details.address, hasCertificate: details.hasCertificate });
    }

    render() {
        if (this.state.address !== '') {
            return (
                <Pane>
                    <Heading size={700}>Device Details</Heading>
                    <Text>Address: {this.state.address}.</Text>
                    <Button intent='none' onClick={this.props.onVerifyClicked} disabled={!this.state.hasCertificate}>Verify The Device Certificate</Button>
                </Pane>
            );
        }

        return (
            <Pane>
                <Spinner />
            </Pane>
        );
    }
}

export default DeviceDetails;
