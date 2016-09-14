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
import ContentCreate from 'material-ui/svg-icons/content/create';

import { Balance, Container, ContainerTitle, Form, InputInline, IdentityIcon } from '../../../ui';

import styles from './header.css';

const DEFAULT_NAME = 'Unnamed';

export default class Header extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    account: PropTypes.object,
    balance: PropTypes.object,
    isTest: PropTypes.bool
  }

  state = {
    name: null
  }

  componentWillMount () {
    this.setName();
  }

  componentWillReceiveProps () {
    this.setName();
  }

  render () {
    const { account, balance } = this.props;
    const { address } = account;
    const { name } = this.state;

    if (!account) {
      return null;
    }

    return (
      <Container>
        <IdentityIcon
          address={ address } />
        <Form>
          <div className={ styles.floatleft }>
            <InputInline
              label='account name'
              hint='a descriptive name for the account'
              value={ name }
              static={ this.renderTitle(name) }
              onSubmit={ this.onSubmitName } />
            <div className={ styles.infoline }>
              { address }
            </div>
            { this.renderTxCount() }
          </div>
          <div className={ styles.balances }>
            <Balance
              account={ account }
              balance={ balance } />
          </div>
        </Form>
      </Container>
    );
  }

  renderTitle (name) {
    return (
      <ContainerTitle title={
        <span>
          <span>{ name || DEFAULT_NAME }</span>
          <ContentCreate
            className={ styles.editicon }
            color='rgb(0, 151, 167)' />
        </span>
      } />
    );
  }

  renderTxCount () {
    const { isTest, balance } = this.props;

    if (!balance) {
      return null;
    }

    const txCount = balance.txCount.sub(isTest ? 0x100000 : 0);

    return (
      <div className={ styles.infoline }>
        { txCount.toFormat() } outgoing transactions
      </div>
    );
  }

  onSubmitName = (name) => {
    const { api } = this.context;
    const { account } = this.props;

    this.setState({ name }, () => {
      api.personal
        .setAccountName(account.address, name)
        .catch((error) => {
          console.error(error);
        });
    });
  }

  setName () {
    const { account } = this.props;

    if (account && account.name !== this.propName) {
      this.propName = account.name;
      this.setState({
        name: account.name
      });
    }
  }
}
