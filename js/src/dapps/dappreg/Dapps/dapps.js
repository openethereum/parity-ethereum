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

import CreateDappCard from '../CreateDappCard';
import DappCard from '../DappCard';

import styles from './dapps.css';

export default class Dapps extends Component {
  static propTypes = {
    dapps: PropTypes.array.isRequired,
    title: PropTypes.string.isRequired,
    own: PropTypes.bool
  };

  static defaultProps = {
    own: false
  };

  render () {
    const { dapps, title } = this.props;

    return (
      <div className={ styles.dapps }>
        <h2 className={ styles.title }>{ title }</h2>
        <div className={ styles.container }>
          { this.renderAddDapp() }
          { this.renderDapps(dapps) }
        </div>
      </div>
    );
  }

  renderAddDapp () {
    const { own } = this.props;

    if (!own) {
      return null;
    }

    return (
      <CreateDappCard />
    );
  }

  renderDapps (dapps) {
    return dapps.map((dapp) => {
      const { id } = dapp;

      return (
        <DappCard
          dapp={ dapp }
          key={ id }
        />
      );
    });
  }
}
