// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import React, { PropTypes } from 'react';
import { connect } from 'react-redux';

import styles from './hash.css';

const leading0x = /^0x/;

const Hash = ({ hash, isTestnet, linked }) => {
  hash = hash.toLowerCase().replace(leading0x, '');
  const shortened = hash.length > (6 + 6)
    ? hash.substr(0, 6) + '...' + hash.slice(-6)
    : hash;

  if (linked) {
    const type = hash.length === 40 ? 'address' : 'tx';
    const url = `https://${isTestnet ? 'testnet.' : ''}etherscan.io/${type}/0x${hash}`;

    return (
      <a
        className={ styles.link }
        href={ url }
        target='_blank'
      >
        <abbr title={ hash }>{ shortened }</abbr>
      </a>
    );
  }

  return (<abbr title={ hash }>{ shortened }</abbr>);
};

Hash.propTypes = {
  hash: PropTypes.string.isRequired,
  isTestnet: PropTypes.bool.isRequired,
  linked: PropTypes.bool
};

Hash.defaultProps = {
  linked: false
};

export default connect(
  (state) => ({ // mapStateToProps
    isTestnet: state.isTestnet
  }),
  null // mapDispatchToProps
)(Hash);
