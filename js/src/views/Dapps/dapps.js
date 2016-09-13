// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { bindActionCreators } from 'redux';

import { Actionbar, Page } from '../../ui';

import Summary from './Summary';

import styles from './dapps.css';

class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    tokens: PropTypes.object
  }

  state = {
    apps: [
      {
        name: 'GAVcoin',
        address: '0x6C5b287A875298f773225e72ce3fA8B2782e0347',
        description: 'Manage your GAVcoins, the hottest new property in crypto',
        url: 'gavcoin'
      },
      {
        name: 'Registry',
        address: '0x8E4e9B13D4b45Cb0befC93c3061b1408f67316B2',
        description: 'A global registry of addresses on the network',
        url: 'registry'
      },
      {
        name: 'Token Registry',
        address: '0x1AE76cf6Ee3955F773C429801a203f08c84B7cc5',
        description: 'A registry of transactable tokens on the network',
        url: 'tokenreg'
      }
    ]
  }

  render () {
    return (
      <div>
        <Actionbar
          title='Decentralized Applications' />
        <Page>
          <div className={ styles.contracts }>
            { this.renderApps() }
          </div>
        </Page>
      </div>
    );
  }

  renderApps () {
    const { tokens } = this.props;
    const { apps } = this.state;

    return apps.map((app, idx) => {
      return (
        <div
          className={ styles.contract }
          key={ app.address }>
          <Summary
            app={ app }
            tokens={ tokens } />
        </div>
      );
    });
  }
}

function mapStateToProps (state) {
  const { tokens } = state.balances;

  return {
    tokens
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Dapps);
