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

import Header from '../Account/Header';
import Transactions from '../Account/Transactions';

import styles from './address.css';

class Address extends Component {
  static propTypes = {
    contacts: PropTypes.object,
    balances: PropTypes.object,
    isTest: PropTypes.bool,
    params: PropTypes.object
  }

  render () {
    const { contacts, balances, isTest } = this.props;
    const { address } = this.props.params;

    const contact = (contacts || {})[address];
    const balance = (balances || {})[address];

    if (!contact) {
      return null;
    }

    return (
      <div className={ styles.address }>
        { this.renderActionbar() }
        <Page>
          <Header
            isTest={ isTest }
            account={ contact }
            balance={ balance } />
          <Transactions
            address={ address } />
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
    ];

    return (
      <Actionbar
        title='Address Information'
        buttons={ buttons } />
    );
  }
}

function mapStateToProps (state) {
  const { contacts } = state.personal;
  const { balances } = state.balances;
  const { isTest } = state.nodeStatus;

  return {
    isTest,
    contacts,
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Address);
