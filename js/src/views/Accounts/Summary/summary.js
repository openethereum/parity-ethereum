import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';

import Balances from '../../../ui/Balances';
import { Container, ContainerTitle, IdentityIcon } from '../../../ui';

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

    return (
      <Container>
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
