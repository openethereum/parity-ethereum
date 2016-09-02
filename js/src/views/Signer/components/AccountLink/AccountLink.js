import React, { Component, PropTypes } from 'react';

import { getAccountLink } from '../util/account';
import styles from './AccountLink.css';

export default class AccountLink extends Component {

  static propTypes = {
    chain: PropTypes.string.isRequired,
    address: PropTypes.string.isRequired,
    className: PropTypes.string,
    children: PropTypes.node
  }

  state = {
    link: null
  };

  componentWillMount () {
    const { address, chain } = this.props;
    this.updateLink(address, chain);
  }

  componentWillReceiveProps (nextProps) {
    const { address, chain } = nextProps;
    this.updateLink(address, chain);
  }

  render () {
    const { children, address, className } = this.props;
    return (
      <a
        href={ this.state.link }
        target='_blank'
        className={ `${styles.container} ${className}` }
        >
        { children || address }
      </a>
    );
  }

  updateLink (address, chain) {
    const link = getAccountLink(address, chain);
    this.setState({ link });
  }

}
