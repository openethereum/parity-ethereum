import React, { Component, PropTypes } from 'react';

import Form, { Input } from '../../../Form';

export default class Verify extends Component {
  static PropTypes = {
    address: PropTypes.string,
    recipient: PropTypes.string
  }

  state = {
    password: ''
  }

  render () {
    return (
      <Form>
        <Input
          value={ this.stats.password }
          type='password' />
      </Form>
    );
  }
}
