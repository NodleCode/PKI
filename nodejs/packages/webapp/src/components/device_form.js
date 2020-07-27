import React from 'react';
import { Button, Heading, Pane, TextInputField } from 'evergreen-ui';

class DeviceForm extends React.Component {
  constructor(props) {
    super(props);
    this.state = { value: '' }
    this.submit = this.submit.bind(this);
  }

  submit(e) {
    e.preventDefault();

    this.props.onSubmit(this.state.value);
  }

  render() {
    return (
      <Pane>
        <Heading
          marginBottom="10px"
          size={700}>
          Enter Target Device
        </Heading>
        <TextInputField
          width="400px"
          placeholder='http://raspberrypi.local:8080'
          label=""
          value={this.state.value}
          onChange={e => this.setState({ value: e.target.value })} />
        <Button
          intent='none'
          onClick={this.submit}>
          Access Device
        </Button>
      </Pane>
    );
  }
}

export default DeviceForm;
