import React from 'react';
import { Pane } from 'evergreen-ui';
import { DeviceForm } from './components';

class App extends React.Component {
  render() {
    return (
      <Pane
        display="flex"
        alignItems="center"
        justifyContent="center"
      >
        <DeviceForm onSubmit={(url) => alert(url)} />
      </ Pane>
    );
  }
}

export default App;
