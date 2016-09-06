import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';
import ActionAccountBalanceWallet from 'material-ui/svg-icons/action/account-balance-wallet';
import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';

import Balances from '../../../ui/Balances';
import { Container, ContainerTitle, IdentityIcon } from '../../../ui';

import styles from './summary.css';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    account: PropTypes.object.isRequired,
    contact: PropTypes.bool,
    children: PropTypes.node
  }

  state = {
    name: 'Unnamed'
  }

  render () {
    const { account, children, contact } = this.props;

    if (!account) {
      return null;
    }

    const viewLink = `/${contact ? 'address' : 'account'}/${account.address}`;
    const typeIcon = contact
      ? <CommunicationContacts />
      : <ActionAccountBalanceWallet />;

    return (
      <Container>
        <div className={ styles.typeIcon }>
          { typeIcon }
        </div>
        <IdentityIcon
          address={ account.address } />
        <ContainerTitle
          title={ <Link to={ viewLink }>{ account.name || 'Unnamed' }</Link> }
          byline={ account.address } />
        <Balances
          account={ account } />
        { children }
      </Container>
    );
  }
}
