import React from 'react';
import { Pane } from 'evergreen-ui';
import { DeviceDetails, DeviceForm } from './components';

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

  onVerifyClicked() {
    console.info('TODO');
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
