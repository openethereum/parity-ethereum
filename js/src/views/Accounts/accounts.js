import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';

import List from './List';
import { CreateAccount } from '../../modals';
import { Actionbar, Page, Tooltip } from '../../ui';

import styles from './accounts.css';

class Accounts extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    accounts: PropTypes.object,
    hasAccounts: PropTypes.bool,
    balances: PropTypes.object
  }

  state = {
    addressBook: false,
    newDialog: false
  }

  render () {
    const { accounts, hasAccounts, balances } = this.props;

    return (
      <div className={ styles.accounts }>
        { this.renderNewDialog() }
        { this.renderActionbar() }
        <Page>
          <List
            accounts={ accounts }
            balances={ balances }
            empty={ !hasAccounts } />
          <Tooltip
            className={ styles.accountTooltip }
            text='your accounts are visible for easy access, allowing you to edit the meta information, make transfers, view transactions and fund the account' />
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
      <FlatButton
        key='newAccount'
        icon={ <ContentAdd /> }
        label='new account'
        primary
        onTouchTap={ this.onNewAccountClick } />
    ];

    return (
      <Actionbar
        className={ styles.toolbar }
        title='Accounts Overview'
        buttons={ buttons }>
        <Tooltip
          className={ styles.toolbarTooltip }
          right
          text='actions relating to the current view are available on the toolbar for quick access, be it for performing actions or creating a new item' />
      </Actionbar>
    );
  }

  renderNewDialog () {
    const { newDialog } = this.state;

    if (!newDialog) {
      return null;
    }

    return (
      <CreateAccount
        onClose={ this.onNewAccountClose }
        onUpdate={ this.onNewAccountUpdate } />
    );
  }

  onNewAccountClick = () => {
    this.setState({
      newDialog: !this.state.newDialog
    });
  }

  onNewAccountClose = () => {
    this.onNewAccountClick();
  }

  onNewAccountUpdate = () => {
  }
}

function mapStateToProps (state) {
  const { accounts, hasAccounts } = state.personal;
  const { balances } = state.balances;

  return {
    accounts,
    hasAccounts,
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Accounts);
