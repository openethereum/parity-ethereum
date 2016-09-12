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
    contacts: PropTypes.object
  }

  state = {
    showAdd: false
  }

  render () {
    const { contacts } = this.props;

    return (
      <div className={ styles.addresses }>
        { this.renderActionbar() }
        { this.renderAddAddress() }
        <Page>
          <List
            contact
            accounts={ contacts } />
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
  const { contacts, hasContacts } = state.personal;

  return {
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
