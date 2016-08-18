import React, { Component, PropTypes } from 'react';

import ContentCreate from 'material-ui/svg-icons/content/create';

import { FundAccount, Transfer } from '../../modals';

import Balances from '../../ui/Balances';
import Container, { Title } from '../../ui/Container';
import Form, { FormWrap, Input, InputInline } from '../../ui/Form';
import IdentityIcon from '../../ui/IdentityIcon';

import Actions from './Actions';
import Transactions from './Transactions';

import styles from './style.css';

export default class Account extends Component {
  static contextTypes = {
    api: React.PropTypes.object,
    accounts: PropTypes.array
  }

  static propTypes = {
    params: PropTypes.object
  }

  state = {
    name: 'Unnamed',
    fundDialog: false,
    transferDialog: false
  }

  componentWillMount () {
    this.retrieveMeta();
  }

  render () {
    const address = this.props.params.address;
    const account = this.context.accounts.find((account) => account.address === address);

    if (!account) {
      return null;
    }

    const title = (
      <span>
        <span>{ this.state.name || 'Unnamed' }</span>
        <ContentCreate
          className={ styles.editicon }
          color='rgb(0, 151, 167)' />
      </span>
    );

    return (
      <div>
        { this.renderFundDialog() }
        { this.renderTransferDialog() }
        <Actions
          onFundAccount={ this.onFundAccountClick }
          onTransfer={ this.onTransferClick } />
        <Container>
          <IdentityIcon
            address={ address } />
          <Form>
            <FormWrap>
              <InputInline
                label='account name'
                hint='a descriptive name for the account'
                value={ this.state.name }
                static={ <Title title={ title } /> }
                onChange={ this.onEditName } />
            </FormWrap>
            <FormWrap>
              <Input
                disabled
                label='account address'
                hint='the account network address'
                value={ address } />
            </FormWrap>
            <FormWrap>
              <Balances
                account={ account }
                onChange={ this.onChangeBalances } />
            </FormWrap>
          </Form>
        </Container>
        <Transactions
          address={ address } />
      </div>
    );
  }

  renderFundDialog () {
    if (!this.state.fundDialog) {
      return null;
    }

    return (
      <FundAccount
        address={ this.props.params.address }
        onClose={ this.onFundAccountClose } />
    );
  }

  renderTransferDialog () {
    if (!this.state.transferDialog) {
      return null;
    }

    const address = this.props.params.address;
    const account = this.context.accounts.find((account) => account.address === address);

    return (
      <Transfer
        account={ account }
        onClose={ this.onTransferClose } />
    );
  }

  onFundAccountClick = () => {
    this.setState({ fundDialog: !this.state.fundDialog });
  }

  onFundAccountClose = () => {
    this.onFundAccountClick();
  }

  onTransferClick = () => {
    this.setState({ transferDialog: !this.state.transferDialog });
  }

  onTransferClose = () => {
    this.onTransferClick();
  }

  onChangeBalances = (balances) => {
    this.setState({
      balances: balances
    });
  }

  onEditName = (event) => {
    const api = this.context.api;
    const name = event.target.value;

    this.setState({
      name: name
    }, () => {
      api.personal.setAccountName(this.props.params.address, name);
    });
  }

  retrieveMeta () {
    this.context.api.personal
      .accountsInfo()
      .then((infos) => {
        const info = infos[this.props.params.address];
        this.setState({
          name: info.name,
          uuid: info.uuid,
          meta: info.meta
        });
      });
  }
}
