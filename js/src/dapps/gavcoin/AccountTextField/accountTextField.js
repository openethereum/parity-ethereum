import React, { Component, PropTypes } from 'react';

export default class AccountTextField extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    account: PropTypes.object
  }

  render () {
    return (
      <div>Account Edit</div>
    );
  }
}
