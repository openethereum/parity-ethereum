import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';

import List from '../Accounts/List';
import { AddAddress } from '../../modals';
import { Actionbar, Page } from '../../ui';

import styles from './addresses.css';

class Addresses extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    balances: PropTypes.object,
    contacts: PropTypes.object,
    hasContacts: PropTypes.bool
  }

  state = {
    showAdd: false
  }

  render () {
    const { balances, contacts, hasContacts } = this.props;

    return (
      <div className={ styles.addresses }>
        { this.renderActionbar() }
        { this.renderAddAddress() }
        <Page>
          <List
            contact
            accounts={ contacts }
            balances={ balances }
            empty={ !hasContacts } />
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
      <FlatButton
        key='newAddress'
        icon={ <ContentAdd /> }
        label='new address'
        primary
        onTouchTap={ this.onOpenAdd } />
    ];

    return (
      <Actionbar
        className={ styles.toolbar }
        title='Saved Addresses'
        buttons={ buttons } />
    );
  }

  renderAddAddress () {
    const { showAdd } = this.state;

    if (!showAdd) {
      return null;
    }

    return (
      <AddAddress
        onClose={ this.onCloseAdd } />
    );
  }

  onOpenAdd = () => {
    this.setState({
      showAdd: true
    });
  }

  onCloseAdd = (address, name, description) => {
    const { api } = this.context;

    this.setState({
      showAdd: false
    });

    if (address) {
      Promise.all([
        api.personal.setAccountName(address, name),
        api.personal.setAccountMeta(address, { description })
      ]).catch((error) => {
        console.error('updateDetails', error);
      });
    }
  }
}

function mapStateToProps (state) {
  const { balances } = state.balances;
  const { contacts, hasContacts } = state.personal;

  return {
    balances,
    contacts,
    hasContacts
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Addresses);
