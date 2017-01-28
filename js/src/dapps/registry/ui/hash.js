// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import etherscanUrl from '../util/etherscan-url';

import styles from './hash.css';

const leading0x = /^0x/;

class Hash extends Component {
  static propTypes = {
    hash: PropTypes.string.isRequired,
    isTestnet: PropTypes.bool.isRequired,
    linked: PropTypes.bool
  }

  static defaultProps = {
    linked: false
  }

  render () {
    const { hash, isTestnet, linked } = this.props;

    let shortened = hash.toLowerCase().replace(leading0x, '');

    shortened = shortened.length > (6 + 6)
      ? shortened.substr(0, 6) + '...' + shortened.slice(-6)
      : shortened;

    if (linked) {
      return (
        <a
          className={ styles.link }
          href={ etherscanUrl(hash, isTestnet) }
          target='_blank'
        >
          <abbr title={ hash }>{ shortened }</abbr>
        </a>
      );
    }

    return (<abbr title={ hash }>{ shortened }</abbr>);
  }
}

export default connect(
  (state) => ({ // mapStateToProps
    isTestnet: state.isTestnet
  }),
  null // mapDispatchToProps
)(Hash);
