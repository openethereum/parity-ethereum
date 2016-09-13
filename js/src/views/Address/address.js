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
