import React, { Component, PropTypes } from 'react';

const { IdentityIcon } = window.parity.react;

export default class ColumnAddress extends Component {
  static contextTypes = {
    accounts: PropTypes.array
  }

  static propTypes = {
    address: PropTypes.string
  }

  render () {
    const { address } = this.props;

    return (
      <td className='account'>
        <IdentityIcon inline center address={ address } />
        { this.renderName() }
      </td>
    );
  }

  renderName () {
    const { address } = this.props;
    const account = this.context.accounts.find((_account) => _account.address === address);

    if (account) {
      return (
        <div className='name'>{ account.name }</div>
      );
    }

    return (
      <div className='address'>{ address }</div>
    );
  }
}
