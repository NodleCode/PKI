import React from 'react';
import { Button, Pane } from 'evergreen-ui'

class App extends React.Component {
  render() {
    return (
      <Pane
        display="flex"
        alignItems="center"
        justifyContent="center"
      >
        <Button>I am using Evergreen!</Button>
      </ Pane>
    );
  }
}

export default App;
