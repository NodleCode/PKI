import React from 'react';
import { Pane, toaster } from 'evergreen-ui';
import { DeviceDetails, DeviceForm } from './components';

import { FirmwareClient } from 'client';
import { Runtime } from 'pki';

class App extends React.Component {
  constructor(props) {
    super(props);
    this.state = { url: '' };
    this.onUrlSubmitted = this.onUrlSubmitted.bind(this);
    this.onVerifyClicked = this.onVerifyClicked.bind(this);
  }

  onUrlSubmitted(url) {
    this.setState({ url: url });
  }

  async onVerifyClicked() {
    const wsRpc = process.env.WS_RPC_URL || 'ws://localhost:9944';
    const runtime = new Runtime(wsRpc);
    await runtime.connect();

    const client = new FirmwareClient(this.state.url);
    try {
      await client.verify(runtime);

      toaster.success('Succesfully verified the device certificate', {
        description: 'In a live production deployment this is when we would negotiate a secure session and continue our operations.',
      });
    } catch (e) {
      toaster.danger('Certificate is not genuine', {
        description: e.toString() + '.',
      });
    }

    // In order to send the user back to form we now clean the state
    this.setState({ url: '' });
  }

  render() {
    let currentComponent = (
      <DeviceForm onSubmit={this.onUrlSubmitted} />
    );

    if (this.state.url.length > 0) {
      currentComponent = (
        <DeviceDetails url={this.state.url} onVerifyClicked={this.onVerifyClicked} />
      )
    }

    return (
      <Pane
        display="flex"
        alignItems="center"
        justifyContent="center"
      >
        {currentComponent}
      </ Pane>
    );
  }
}

export default App;
