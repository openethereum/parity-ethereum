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

import { sha3 } from '../../api/util/sha3';
import Contracts from '../../contracts';
import { setAddressImage } from '../../redux/actions';
import { Actionbar, Page } from '../../ui';

import Summary from './Summary';

import styles from './dapps.css';

const APPS = [
  {
    name: 'Token Deployment',
    description: 'Deploy new basic tokens that you are able to send around',
    url: 'basiccoin'
  },
  {
    name: 'GAVcoin',
    description: 'Manage your GAVcoins, the hottest new property in crypto',
    url: 'gavcoin'
  },
  {
    name: 'Registry',
    description: 'A global registry of addresses on the network',
    url: 'registry'
  },
  {
    name: 'Token Registry',
    description: 'A registry of transactable tokens on the network',
    url: 'tokenreg'
  },
  {
    name: 'Method Registry',
    description: 'A registry of method signatures for lookups on transactions',
    url: 'signaturereg'
  },
  {
    name: 'GitHub Hint',
    description: 'A mapping of GitHub URLs to hashes for use in contracts as references',
    url: 'githubhint'
  }
];

APPS.forEach((app) => {
  app.id = sha3(app.url);
  console.log(`dapps ${app.id} -> ${app.url}`);
});

class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    tokens: PropTypes.object,
    setAddressImage: PropTypes.func.isRequired
  }

  state = {
    apps: APPS
  }

  componentDidMount () {
    this.loadImages();
  }

  render () {
    return (
      <div>
        <Actionbar
          title='Decentralized Applications' />
        <Page>
          <div className={ styles.list }>
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
          className={ styles.item }
          key={ app.url }>
          <Summary
            app={ app }
            tokens={ tokens } />
        </div>
      );
    });
  }

  loadImages () {
    const { setAddressImage } = this.props;
    const { apps } = this.state;
    const { dappReg } = Contracts.get();

    Promise
      .all(apps.map((app) => dappReg.getImage(app.id)))
      .then((images) => {
        apps.forEach((app, index) => {
          setAddressImage(app.id, images[index]);
        });
        this.setState({ apps });
      })
      .catch((error) => {
        console.error('loadImages', error);
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
  return bindActionCreators({ setAddressImage }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Dapps);
