// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import { observe } from 'mobx';
import { observer } from 'mobx-react';
import { uniq, isEqual, pickBy } from 'lodash';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { Link } from 'react-router';
import { bindActionCreators } from 'redux';

import HardwareStore from '~/mobx/hardwareStore';
import { CreateAccount, CreateWallet } from '~/modals';
import { Actionbar, ActionbarExport, ActionbarSearch, ActionbarSort, Button, Page, Tooltip } from '~/ui';
import { AddIcon, KeyIcon } from '~/ui/Icons';
import { setVisibleAccounts } from '~/redux/providers/personalActions';

import List from './List';
import styles from './accounts.css';

@observer
class Accounts extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    accountsInfo: PropTypes.object.isRequired,
    balances: PropTypes.object,
    hasAccounts: PropTypes.bool.isRequired,
    setVisibleAccounts: PropTypes.func.isRequired
  }

  hwstore = HardwareStore.get(this.context.api);

  state = {
    _observeCancel: null,
    addressBook: false,
    newDialog: false,
    newWalletDialog: false,
    sortOrder: '',
    searchValues: [],
    searchTokens: [],
    show: false
  }

  componentWillMount () {
    // FIXME: Messy, figure out what it fixes and do it elegantly
    window.setTimeout(() => {
      this.setState({ show: true });
    }, 100);

    this.setVisibleAccounts();

    this.setState({
      _observeCancel: observe(this.hwstore, 'wallets', this.onHardwareChange, true)
    });
  }

  componentWillReceiveProps (nextProps) {
    const prevAddresses = Object.keys(this.props.accounts);
    const nextAddresses = Object.keys(nextProps.accounts);

    if (prevAddresses.length !== nextAddresses.length || !isEqual(prevAddresses.sort(), nextAddresses.sort())) {
      this.setVisibleAccounts(nextProps);
    }
  }

  componentWillUnmount () {
    this.props.setVisibleAccounts([]);
    this.state._observeCancel();
  }

  setVisibleAccounts (props = this.props) {
    const { accounts, setVisibleAccounts } = props;

    setVisibleAccounts(Object.keys(accounts));
  }

  render () {
    return (
      <div>
        { this.renderNewDialog() }
        { this.renderNewWalletDialog() }
        { this.renderActionbar() }

        <Page>
          <Tooltip
            className={ styles.accountTooltip }
            text={
              <FormattedMessage
                id='accounts.tooltip.overview'
                defaultMessage='your accounts are visible for easy access, allowing you to edit the meta information, make transfers, view transactions and fund the account'
              />
            }
          />

          { this.renderHwWallets() }
          { this.renderWallets() }
          { this.renderAccounts() }
        </Page>
      </div>
    );
  }

  renderLoading (object) {
    const loadings = ((object && Object.keys(object)) || []).map((_, idx) => (
      <div key={ idx } className={ styles.loading }>
        <div />
      </div>
    ));

    return (
      <div className={ styles.loadings }>
        { loadings }
      </div>
    );
  }

  renderAccounts () {
    const { accounts, balances } = this.props;
    const _accounts = pickBy(accounts, (account) => account.uuid);
    const _hasAccounts = Object.keys(_accounts).length > 0;

    if (!this.state.show) {
      return this.renderLoading(_accounts);
    }

    const { searchValues, sortOrder } = this.state;

    return (
      <List
        search={ searchValues }
        accounts={ _accounts }
        balances={ balances }
        empty={ !_hasAccounts }
        order={ sortOrder }
        handleAddSearchToken={ this.onAddSearchToken }
      />
    );
  }

  renderWallets () {
    const { accounts, balances } = this.props;
    const wallets = pickBy(accounts, (account) => account.wallet);
    const hasWallets = Object.keys(wallets).length > 0;

    if (!hasWallets) {
      return null;
    }

    if (!this.state.show) {
      return this.renderLoading(wallets);
    }

    const { searchValues, sortOrder } = this.state;

    return (
      <List
        link='wallet'
        search={ searchValues }
        accounts={ wallets }
        balances={ balances }
        order={ sortOrder }
        handleAddSearchToken={ this.onAddSearchToken }
      />
    );
  }

  renderHwWallets () {
    const { accounts, balances } = this.props;
    const { wallets } = this.hwstore;
    const hardware = pickBy(accounts, (account) => account.hardware);
    const hasHardware = Object.keys(hardware).length > 0;

    if (!hasHardware) {
      return null;
    }

    if (!this.state.show) {
      return this.renderLoading(hardware);
    }

    const { searchValues, sortOrder } = this.state;
    const disabled = Object
      .keys(hardware)
      .filter((address) => !wallets[address])
      .reduce((result, address) => {
        result[address] = true;
        return result;
      }, {});

    return (
      <List
        search={ searchValues }
        accounts={ hardware }
        balances={ balances }
        disabled={ disabled }
        order={ sortOrder }
        handleAddSearchToken={ this.onAddSearchToken }
      />
    );
  }

  renderSearchButton () {
    const onChange = (searchTokens, searchValues) => {
      this.setState({ searchTokens, searchValues });
    };

    return (
      <ActionbarSearch
        key='searchAccount'
        tokens={ this.state.searchTokens }
        onChange={ onChange }
      />
    );
  }

  renderSortButton () {
    const onChange = (sortOrder) => {
      this.setState({ sortOrder });
    };

    return (
      <ActionbarSort
        key='sortAccounts'
        id='sortAccounts'
        order={ this.state.sortOrder }
        onChange={ onChange }
      />
    );
  }

  renderActionbar () {
    const { accounts } = this.props;

    const buttons = [
      <Link
        to='/vaults'
        key='vaults'
      >
        <Button
          icon={ <KeyIcon /> }
          label={
            <FormattedMessage
              id='accounts.button.vaults'
              defaultMessage='vaults'
            />
          }
          onClick={ this.onVaultsClick }
        />
      </Link>,
      <Button
        key='newAccount'
        icon={ <AddIcon /> }
        label={
          <FormattedMessage
            id='accounts.button.newAccount'
            defaultMessage='account'
          />
        }
        onClick={ this.onNewAccountClick }
      />,
      <Button
        key='newWallet'
        icon={ <AddIcon /> }
        label={
          <FormattedMessage
            id='accounts.button.newWallet'
            defaultMessage='wallet'
          />
        }
        onClick={ this.onNewWalletClick }
      />,
      <ActionbarExport
        key='exportAccounts'
        content={ accounts }
        filename='accounts'
      />,
      this.renderSearchButton(),
      this.renderSortButton()
    ];

    return (
      <Actionbar
        className={ styles.toolbar }
        title={
          <FormattedMessage
            id='accounts.title'
            defaultMessage='Accounts Overview'
          />
        }
        buttons={ buttons }
      >
        <Tooltip
          className={ styles.toolbarTooltip }
          right
          text={
            <FormattedMessage
              id='accounts.tooltip.actions'
              defaultMessage='actions relating to the current view are available on the toolbar for quick access, be it for performing actions or creating a new item'
            />
          }
        />
      </Actionbar>
    );
  }

  renderNewDialog () {
    const { accounts } = this.props;
    const { newDialog } = this.state;

    if (!newDialog) {
      return null;
    }

    return (
      <CreateAccount
        accounts={ accounts }
        onClose={ this.onNewAccountClose }
        onUpdate={ this.onNewAccountUpdate }
      />
    );
  }

  renderNewWalletDialog () {
    const { accounts } = this.props;
    const { newWalletDialog } = this.state;

    if (!newWalletDialog) {
      return null;
    }

    return (
      <CreateWallet
        accounts={ accounts }
        onClose={ this.onNewWalletClose }
      />
    );
  }

  onAddSearchToken = (token) => {
    const { searchTokens } = this.state;
    const newSearchTokens = uniq([].concat(searchTokens, token));

    this.setState({ searchTokens: newSearchTokens });
  }

  onNewAccountClick = () => {
    this.setState({
      newDialog: true
    });
  }

  onNewWalletClick = () => {
    this.setState({
      newWalletDialog: true
    });
  }

  onNewAccountClose = () => {
    this.setState({
      newDialog: false
    });
  }

  onNewWalletClose = () => {
    this.setState({
      newWalletDialog: false
    });
  }

  onNewAccountUpdate = () => {
  }

  onHardwareChange = () => {
    const { accountsInfo } = this.props;
    const { wallets } = this.hwstore;

    Object
      .keys(wallets)
      .filter((address) => !accountsInfo[address])
      .forEach((address) => this.hwstore.createAccountInfo(wallets[address]));

    this.setVisibleAccounts();
  }
}

function mapStateToProps (state) {
  const { accounts, accountsInfo, hasAccounts } = state.personal;
  const { balances } = state.balances;

  return {
    accounts,
    accountsInfo,
    balances,
    hasAccounts
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    setVisibleAccounts
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Accounts);
