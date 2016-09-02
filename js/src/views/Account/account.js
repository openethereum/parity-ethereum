import React, { Component, PropTypes } from 'react';
import { FlatButton } from 'material-ui';
import ActionAccountBalance from 'material-ui/svg-icons/action/account-balance';
import ContentCreate from 'material-ui/svg-icons/content/create';
import ContentSend from 'material-ui/svg-icons/content/send';

import { FundAccount, Transfer } from '../../modals';
import { Actionbar, Balances, Container, ContainerTitle, Form, FormWrap, InputInline, IdentityIcon } from '../../ui';

import Transactions from './Transactions';

import styles from './account.css';

const DEFAULT_NAME = 'Unnamed';

export default class Account extends Component {
  static contextTypes = {
    api: React.PropTypes.object,
    accounts: PropTypes.array
  }

  static propTypes = {
    params: PropTypes.object
  }

  propName = null

  state = {
    name: null,
    fundDialog: false,
    transferDialog: false
  }

  componentWillMount () {
    this.setName();
  }

  componentWillReceiveProps () {
    this.setName();
  }

  render () {
    const { address } = this.props.params;
    const { name } = this.state;
    const account = this.context.accounts.find((account) => account.address === address);

    if (!account) {
      return null;
    }

    const title = (
      <span>
        <span>{ name || DEFAULT_NAME }</span>
        <ContentCreate
          className={ styles.editicon }
          color='rgb(0, 151, 167)' />
      </span>
    );

    return (
      <div>
        { this.renderFundDialog() }
        { this.renderTransferDialog() }
        { this.renderActionbar() }
        <Container>
          <IdentityIcon
            address={ address } />
          <Form>
            <div
              className={ styles.floatleft }>
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
                { account.txCount.toFormat() } outgoing transactions
              </div>
            </div>
            <div
              className={ styles.balances }>
              <Balances
                account={ account }
                onChange={ this.onChangeBalances } />
            </div>
          </Form>
        </Container>
        <Transactions
          address={ address } />
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
      <FlatButton
        key='transferFunds'
        icon={ <ContentSend /> }
        label='transfer'
        primary
        onTouchTap={ this.onTransferClick } />,
      <FlatButton
        key='fundAccount'
        icon={ <ActionAccountBalance /> }
        label='fund account'
        primary
        onTouchTap={ this.onFundAccountClick } />
    ];

    return (
      <Actionbar
        title='Account Management'
        buttons={ buttons } />
    );
  }

  renderFundDialog () {
    const { fundDialog } = this.state;

    if (!fundDialog) {
      return null;
    }

    const { address } = this.props.params;

    return (
      <FundAccount
        address={ address }
        onClose={ this.onFundAccountClose } />
    );
  }

  renderTransferDialog () {
    const { transferDialog } = this.state;

    if (!transferDialog) {
      return null;
    }

    const { address } = this.props.params;
    const account = this.context.accounts.find((_account) => _account.address === address);

    return (
      <Transfer
        account={ account }
        onClose={ this.onTransferClose } />
    );
  }

  onFundAccountClick = () => {
    this.setState({
      fundDialog: !this.state.fundDialog
    });
  }

  onFundAccountClose = () => {
    this.onFundAccountClick();
  }

  onTransferClick = () => {
    this.setState({
      transferDialog: !this.state.transferDialog
    });
  }

  onTransferClose = () => {
    this.onTransferClick();
  }

  onChangeBalances = (balances) => {
    this.setState({
      balances: balances
    });
  }

  onEditName = (event, name) => {
    const { api } = this.context;
    const { address } = this.props.params;

    this.setState({ name }, () => {
      api.personal
        .setAccountName(address, name)
        .catch((error) => {
          console.error(error);
        });
    });
  }

  setName () {
    const { address } = this.props.params;
    const account = this.context.accounts.find((account) => account.address === address);

    if (account && account.name !== this.propName) {
      this.propName = account.name;
      this.setState({
        name: account.name
      });
    }
  }
}
