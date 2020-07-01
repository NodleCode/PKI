import React from 'react';
import { Button, Heading, Pane, Spinner, Text, KeyIcon } from 'evergreen-ui';

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
                <Pane
                  display="flex"
                  flexDirection="column"
                  alignItems="flex-start">
                  <Heading 
                    size={700}>
                    Device Address
                  </Heading>
                  <Pane
                    marginY="12px;">
                    <KeyIcon 
                      marginRight="10px"/>
                    <Text>
                      {this.state.address}
                    </Text>
                  </Pane>
                  
                  <Button 
                    borderRadius="5px"
                    marginTop="12px"
                    intent='default' 
                    onClick={this.props.onVerifyClicked} 
                    disabled={!this.state.hasCertificate}>
                    Verify The Device Certificate
                  </Button>
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
