import React, { Component, PropTypes } from 'react';

export default class Accounts extends Component {
  static contextTypes = {
    api: PropTypes.object,
    instance: PropTypes.object
  }

  render () {
    return (
      <div className='accounts'>
        Accounts and balances to display here
      </div>
    );
  }
}
