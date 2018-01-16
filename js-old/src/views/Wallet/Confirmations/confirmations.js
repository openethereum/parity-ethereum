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

import { LinearProgress, MenuItem, IconMenu } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import ReactTooltip from 'react-tooltip';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { confirmOperation, revokeOperation } from '~/redux/providers/walletActions';
import { bytesToHex } from '@parity/api/lib/util/format';
import { Container, InputAddress, Button, IdentityIcon } from '~/ui';
import TxRow from '~/ui/TxList/TxRow';

import styles from '../wallet.css';
import txListStyles from '~/ui/TxList/txList.css';

class WalletConfirmations extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    owners: PropTypes.array.isRequired,
    require: PropTypes.object.isRequired,
    confirmOperation: PropTypes.func.isRequired,
    revokeOperation: PropTypes.func.isRequired,

    confirmations: PropTypes.array
  };

  static defaultProps = {
    confirmations: []
  };

  render () {
    return (
      <div>
        <Container title='Pending Confirmations'>
          { this.renderConfirmations() }
        </Container>
      </div>
    );
  }
  renderConfirmations () {
    const { confirmations, ...others } = this.props;
    const realConfirmations = confirmations && confirmations
      .filter((conf) => conf.confirmedBy.length > 0);

    if (!realConfirmations) {
      return null;
    }

    if (realConfirmations.length === 0) {
      return (
        <div>
          <p>
            <FormattedMessage
              id='wallet.confirmations.none'
              defaultMessage='No transactions needs confirmation right now.'
            />
          </p>
        </div>
      );
    }

    return realConfirmations
      .map((confirmation) => (
        <WalletConfirmation
          key={ confirmation.operation }
          confirmation={ confirmation }
          { ...others }
        />
      ));
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return { accounts };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    confirmOperation,
    revokeOperation
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(WalletConfirmations);

class WalletConfirmation extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    confirmation: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    owners: PropTypes.array.isRequired,
    require: PropTypes.object.isRequired,
    confirmOperation: PropTypes.func.isRequired,
    revokeOperation: PropTypes.func.isRequired
  };

  state = {
    openConfirm: false,
    openRevoke: false
  };

  render () {
    const { confirmation } = this.props;
    const confirmationsRows = [];

    const className = styles.light;

    const txRow = this.renderTransactionRow(confirmation, className);
    const detailsRow = this.renderConfirmedBy(confirmation, className);
    const progressRow = this.renderProgress(confirmation, className);
    const actionsRow = this.renderActions(confirmation, className);

    confirmationsRows.push(progressRow);
    confirmationsRows.push(detailsRow);
    confirmationsRows.push(txRow);
    confirmationsRows.push(actionsRow);

    return (
      <div className={ styles.confirmationContainer }>
        <table className={ [ txListStyles.transactions, styles.confirmations ].join(' ') }>
          <tbody>
            { confirmationsRows }
          </tbody>
        </table>
        { this.renderPending() }
      </div>
    );
  }

  renderPending () {
    const { pending } = this.props.confirmation;

    if (!pending) {
      return null;
    }

    return (
      <div className={ styles.pendingOverlay } />
    );
  }

  handleOpenConfirm = () => {
    this.setState({
      openConfirm: true
    });
  }

  handleCloseConfirm = () => {
    this.setState({
      openConfirm: false
    });
  }

  handleOpenRevoke = () => {
    this.setState({
      openRevoke: true
    });
  }

  handleCloseRevoke = () => {
    this.setState({
      openRevoke: false
    });
  }

  handleConfirm = (e, item) => {
    const { confirmOperation, confirmation, address } = this.props;
    const owner = item.props.value;

    confirmOperation(address, owner, confirmation.operation);
  }

  handleRevoke = (e, item) => {
    const { revokeOperation, confirmation, address } = this.props;
    const owner = item.props.value;

    revokeOperation(address, owner, confirmation.operation);
  }

  renderActions (confirmation, className) {
    const { owners, accounts } = this.props;
    const { operation, confirmedBy, pending } = confirmation;
    const { openConfirm, openRevoke } = this.state;

    const addresses = Object.keys(accounts);

    const possibleConfirm = owners
      .filter((owner) => addresses.includes(owner))
      .filter((owner) => !confirmedBy.includes(owner));

    const possibleRevoke = owners
      .filter((owner) => addresses.includes(owner))
      .filter((owner) => confirmedBy.includes(owner));

    const confirmButton = (
      <Button
        onClick={ this.handleOpenConfirm }
        label={
          <FormattedMessage
            id='wallet.confirmations.buttons.confirmAs'
            defaultMessage='Confirm As...'
          />
        }
        disabled={ pending || possibleConfirm.length === 0 }
      />
    );

    const revokeButton = (
      <Button
        onClick={ this.handleOpenRevoke }
        label={
          <FormattedMessage
            id='wallet.confirmations.buttons.revokeAs'
            defaultMessage='Revoke As...'
          />
        }
        disabled={ pending || possibleRevoke.length === 0 }
      />
    );

    return (
      <tr
        className={ className }
        key={ `actions_${operation}` }
      >
        <td />
        <td colSpan={ 3 }>
          <div className={ styles.actions }>
            <IconMenu
              iconButtonElement={ confirmButton }
              open={ openConfirm }
              onRequestChange={ this.handleCloseConfirm }
              onItemTouchTap={ this.handleConfirm }
            >
              { possibleConfirm.map((address) => this.renderAccountItem(address)) }
            </IconMenu>

            <IconMenu
              iconButtonElement={ revokeButton }
              open={ openRevoke }
              onRequestChange={ this.handleCloseRevoke }
              onItemTouchTap={ this.handleRevoke }
            >
              { possibleRevoke.map((address) => this.renderAccountItem(address)) }
            </IconMenu>
          </div>
        </td>
        <td />
      </tr>
    );
  }

  renderAccountItem (address) {
    const account = this.props.accounts[address];

    return (
      <MenuItem
        key={ address }
        value={ address }
      >
        <div className={ styles.accountItem }>
          <IdentityIcon
            address={ address }
            center
            inline
          />
          <span>{ account.name.toUpperCase() || account.address }</span>
        </div>
      </MenuItem>
    );
  }

  renderProgress (confirmation) {
    const { require } = this.props;
    const { operation, confirmedBy, pending } = confirmation;

    const style = { borderRadius: 0 };

    return (
      <tr key={ `prog_${operation}` }>
        <td
          colSpan={ 5 }
          style={ { padding: 0, paddingTop: '1em' } }
        >
          <div
            data-tip
            data-for={ `tooltip_${operation}` }
            data-effect='solid'
          >
            {
              pending
              ? (
                <LinearProgress
                  key={ `pending_${operation}` }
                  mode='indeterminate'
                  style={ style }
                />
              )
              : (
                <LinearProgress
                  key={ `unpending_${operation}` }
                  mode='determinate'
                  min={ 0 }
                  max={ require.toNumber() }
                  value={ confirmedBy.length }
                  style={ style }
                />
              )
            }
          </div>

          <ReactTooltip id={ `tooltip_${operation}` }>
            <FormattedMessage
              id='wallet.confirmations.tooltip.confirmed'
              defaultMessage='Confirmed by {number}/{required} owners'
              values={ {
                required: require.toNumber(),
                number: confirmedBy.length
              } }
            />
          </ReactTooltip>
        </td>
      </tr>
    );
  }

  renderTransactionRow (confirmation, className) {
    const { address, netVersion } = this.props;
    const { operation, transactionHash, blockNumber, value, to, data } = confirmation;

    if (value && to && data) {
      return (
        <TxRow
          address={ address }
          className={ className }
          historic={ false }
          netVersion={ netVersion }
          key={ operation }
          tx={ {
            hash: transactionHash,
            blockNumber,
            from: address,
            to,
            value,
            input: bytesToHex(data)
          } }
        />
      );
    }

    return (
      <tr
        className={ className }
        key={ operation }
      >
        <td colSpan={ 5 }>
          <code>{ operation }</code>
        </td>
      </tr>
    );
  }

  renderConfirmedBy (confirmation, className) {
    const { operation, confirmedBy } = confirmation;

    const confirmed = confirmedBy.map((owner) => (
      <InputAddress
        key={ owner }
        value={ owner }
        allowCopy={ false }
        hideUnderline
        disabled
        small
        text
      />
    ));

    return (
      <tr key={ `details_${operation}` } className={ className }>
        <td colSpan={ 5 } style={ { padding: 0 } }>
          <div
            data-tip
            data-for={ `tooltip_${operation}` }
            data-effect='solid'
            className={ styles.confirmed }
          >
            { confirmed }
          </div>
        </td>
      </tr>
    );
  }
}
