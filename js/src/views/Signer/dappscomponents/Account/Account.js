import React, { Component, PropTypes } from 'react';

import styles from './Account.css';

import Identicon from '../Identicon';
import AccountLink from '../AccountLink';

export default class Account extends Component {

  static propTypes = {
    className: PropTypes.string,
    address: PropTypes.string.isRequired,
    chain: PropTypes.string.isRequired,
    balance: PropTypes.object, // eth BigNumber, not required since it mght take time to fetch
    name: PropTypes.string
  };

  state = {
    balanceDisplay: '?'
  };

  componentWillMount () {
    this.updateBalanceDisplay(this.props.balance);
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.balance === this.props.balance) {
      return;
    }
    this.updateBalanceDisplay(nextProps.balance);
  }

  updateBalanceDisplay (balance) {
    this.setState({
      balanceDisplay: balance ? balance.div(1e18).toFormat(3) : '?'
    });
  }

  render () {
    const { address, chain, className } = this.props;
    return (
      <div className={ `${styles.acc} ${className}` } title={ this.renderTitle() }>
        <Identicon address={ address } chain={ chain } />
        { this.renderName() }
        { this.renderBalance() }
      </div>
    );
  }

  renderTitle () {
    const { name, address } = this.props;
    if (name) {
      return address + ' ' + name;
    }

    return address;
  }

  renderBalance () {
    const { balanceDisplay } = this.state;
    return (
      <span> <strong>{ balanceDisplay }</strong> <small>ETH</small></span>
    );
  }

  renderName () {
    const { address, name } = this.props;
    if (!name) {
      return (
        <AccountLink address={ address } chain={ this.props.chain }>
          [{ this.shortAddress(address) }]
        </AccountLink>
      );
    }
    return (
      <AccountLink address={ address } chain={ this.props.chain } >
        <span>
          <span className={ styles.name }>{ name }</span>
          <span className={ styles.address }>[{ this.tinyAddress(address) }]</span>
        </span>
      </AccountLink>
    );
  }

  tinyAddress () {
    const { address } = this.props;
    const len = address.length;
    return address.slice(2, 4) + '..' + address.slice(len - 2);
  }

  shortAddress () {
    const { address } = this.props;
    const len = address.length;
    return address.slice(2, 8) + '..' + address.slice(len - 7);
  }

}
