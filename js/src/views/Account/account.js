import React, { Component, PropTypes } from 'react';

import ContentCreate from 'material-ui/svg-icons/content/create';

import { FundAccount, Transfer } from '../../modals';

import Balances from '../../ui/Balances';
import Container, { Title } from '../../ui/Container';
import Form, { FormWrap, InputInline } from '../../ui/Form';
import IdentityIcon from '../../ui/IdentityIcon';

import Actions from './Actions';
import Transactions from './Transactions';

import styles from './style.css';

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
                value={ name }
                static={ <Title title={ title } /> }
                onChange={ this.onEditName } />
            </FormWrap>
            <FormWrap>
              <div className={ styles.infoline }>
                { address }
              </div>
              <div className={ styles.infoline }>
                { account.txCount.toFormat() } outgoing transactions
              </div>
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
