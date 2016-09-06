import React, { Component, PropTypes } from 'react';
import ContentCreate from 'material-ui/svg-icons/content/create';

import { Balances, Container, ContainerTitle, Form, InputInline, IdentityIcon } from '../../../ui';

import styles from './header.css';

const DEFAULT_NAME = 'Unnamed';

export default class Header extends Component {
  static contextTypes = {
    api: PropTypes.object,
    balances: PropTypes.object
  }

  static propTypes = {
    account: PropTypes.object
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
    const { balances } = this.context;
    const { account } = this.props;
    const { address } = account;
    const { name } = this.state;
    const balance = balances[address];

    const title = (
      <span>
        <span>{ name || DEFAULT_NAME }</span>
        <ContentCreate
          className={ styles.editicon }
          color='rgb(0, 151, 167)' />
      </span>
    );

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
              static={ <ContainerTitle title={ title } /> }
              onChange={ this.onEditName } />
            <div className={ styles.infoline }>
              { address }
            </div>
            <div className={ styles.infoline }>
              { balance.txCount.toFormat() } outgoing transactions
            </div>
          </div>
          <div className={ styles.balances }>
            <Balances
              account={ account } />
          </div>
        </Form>
      </Container>
    );
  }

  onEditName = (event, name) => {
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
