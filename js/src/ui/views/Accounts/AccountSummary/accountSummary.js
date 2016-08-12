import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';

import { CardTitle } from 'material-ui/Card';

import Balances from '../../../Balances';
import Container from '../../../Container';
import IdentityIcon from '../../../IdentityIcon';

const TITLE_STYLE = { textTransform: 'uppercase', paddingBottom: 0, paddingTop: 0 };

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
        <CardTitle
          style={ TITLE_STYLE }
          title={ <Link to={ viewLink }>{ account.name || 'Unnamed' }</Link> } />
        <Balances
          address={ account.address } />
      </Container>
    );
  }
}
