import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';

import Balances from '../../../Balances';
import Container, { Title } from '../../../Container';
import IdentityIcon from '../../../IdentityIcon';

export default class AccountSummary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    account: PropTypes.object.isRequired
  }

  state = {
    name: 'Unnamed'
  }

  render () {
    const account = this.props.account;
    const viewLink = `/account/${account.address}`;

    return (
      <Container>
        <IdentityIcon
          address={ account.address } />
        <Title
          title={ <Link to={ viewLink }>{ account.name || 'Unnamed' }</Link> }
          byline={ account.address } />
        <Balances
          address={ account.address } />
      </Container>
    );
  }
}
