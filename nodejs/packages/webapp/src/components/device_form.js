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
                <Heading size={700}>Choose a Target Device</Heading>
                <TextInputField
                    placeholder='http://raspberrypi.local:8080'
                    label='Device Url'
                    value={this.state.value}
                    onChange={e => this.setState({ value: e.target.value })} />
                <Button intent='success' onClick={this.submit}>Access Device</Button>
            </Pane>
        );
    }
}

export default DeviceForm;
