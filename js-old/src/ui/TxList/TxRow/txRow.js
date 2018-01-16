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

import moment from 'moment';
import dateDifference from 'date-difference';
import { FormattedMessage } from 'react-intl';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { Link } from 'react-router';

import { txLink } from '~/3rdparty/etherscan/links';

import IdentityIcon from '~/ui/IdentityIcon';
import IdentityName from '~/ui/IdentityName';
import MethodDecoding from '~/ui/MethodDecoding';
import MethodDecodingStore from '~/ui/MethodDecoding/methodDecodingStore';

import styles from '../txList.css';

class TxRow extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accountAddresses: PropTypes.array.isRequired,
    address: PropTypes.string.isRequired,
    blockNumber: PropTypes.object,
    contractAddresses: PropTypes.array.isRequired,
    netVersion: PropTypes.string.isRequired,
    tx: PropTypes.object.isRequired,

    block: PropTypes.object,
    className: PropTypes.string,
    cancelTransaction: PropTypes.func,
    editTransaction: PropTypes.func,
    historic: PropTypes.bool,
    killTransaction: PropTypes.func
  };

  static defaultProps = {
    historic: true
  };

  state = {
    isCancelOpen: false,
    isEditOpen: false,
    canceled: false,
    editing: false,
    isContract: false,
    isDeploy: false
  };

  methodDecodingStore = MethodDecodingStore.get(this.context.api);

  componentWillMount () {
    const { address, tx } = this.props;

    this
      .methodDecodingStore
      .lookup(address, tx)
      .then((lookup) => {
        const newState = {
          isContract: lookup.contract,
          isDeploy: lookup.deploy
        };

        this.setState(newState);
      });
  }

  render () {
    const { address, className, historic, netVersion, tx } = this.props;

    return (
      <tr className={ className || '' }>
        { this.renderBlockNumber(tx.blockNumber) }
        { this.renderAddress(tx.from, false) }
        <td className={ styles.transaction }>
          { this.renderEtherValue(tx.value) }
          <div>⇒</div>
          <div>
            <a
              className={ styles.link }
              href={ txLink(tx.hash, false, netVersion) }
              target='_blank'
            >
              { `${tx.hash.substr(2, 6)}...${tx.hash.slice(-6)}` }
            </a>
          </div>
        </td>
        { this.renderAddress(tx.to || tx.creates, !!tx.creates) }
        <td className={ styles.method }>
          <MethodDecoding
            historic={ historic }
            address={ address }
            transaction={ tx }
          />
        </td>
      </tr>
    );
  }

  renderAddress (address, isDeploy = false) {
    const isKnownContract = this.getIsKnownContract(address);
    let esLink = null;

    if (address && (!isDeploy || isKnownContract)) {
      esLink = (
        <Link
          activeClassName={ styles.currentLink }
          className={ styles.link }
          to={ this.addressLink(address) }
        >
          <IdentityName
            address={ address }
            shorten
          />
        </Link>
      );
    }

    return (
      <td className={ styles.address }>
        <div className={ styles.center }>
          <IdentityIcon
            center
            className={ styles.icon }
            address={ (!isDeploy || isKnownContract) ? address : '' }
          />
        </div>
        <div className={ styles.center }>
          { esLink || 'DEPLOY' }
        </div>
      </td>
    );
  }

  renderEtherValue (_value) {
    const { api } = this.context;
    const { isContract, isDeploy } = this.state;

    // Always show the value if ETH transfer, ie. not
    // a contract or a deployment
    const fullValue = !(isContract || isDeploy);
    const value = api.util.fromWei(_value);

    if (value.eq(0) && !fullValue) {
      return <div className={ styles.value }>{ ' ' }</div>;
    }

    return (
      <div className={ styles.value }>
        { value.toFormat(5) }<small>ETH</small>
      </div>
    );
  }

  renderBlockNumber (_blockNumber) {
    const { block } = this.props;
    const blockNumber = _blockNumber.toNumber();

    return (
      <td className={ styles.timestamp }>
        <div>{ blockNumber && block ? moment(block.timestamp).fromNow() : null }</div>
        <div>{ blockNumber ? _blockNumber.toFormat() : this.renderCancelToggle() }</div>
      </td>
    );
  }

  renderCancelToggle () {
    const { canceled, editing, isCancelOpen, isEditOpen } = this.state;

    if (canceled) {
      return (
        <div className={ styles.pending }>
          <FormattedMessage
            lassName={ styles.uppercase }
            id='ui.txList.txRow.canceled'
            defaultMessage='Canceled'
          />
        </div>
      );
    }

    if (editing) {
      return (
        <div className={ styles.pending }>
          <div className={ styles.uppercase }>
            <FormattedMessage
              id='ui.txList.txRow.editing'
              defaultMessage='Editing'
            />
          </div>
        </div>
      );
    }

    if (!isCancelOpen && !isEditOpen) {
      const pendingStatus = this.getCondition();
      const isPending = pendingStatus === 'pending';

      return (
        <div className={ styles.pending }>
          {
            isPending
            ? (
              <div className={ styles.pending }>
                <div />
                <div className={ styles.uppercase }>
                  <FormattedMessage
                    id='ui.txList.txRow.submitting'
                    defaultMessage='Pending'
                  />
                </div>
              </div>
            ) : (
              <div>
                <span>
                  { pendingStatus }
                </span>
                <div className={ styles.uppercase }>
                  <FormattedMessage
                    id='ui.txList.txRow.scheduled'
                    defaultMessage='Scheduled'
                  />
                </div>
              </div>
            )
          }
          <a onClick={ this.setEdit } className={ styles.uppercase }>
            <FormattedMessage
              id='ui.txList.txRow.edit'
              defaultMessage='Edit'
            />
          </a>
          <span>{' | '}</span>
          <a onClick={ this.setCancel } className={ styles.uppercase }>
            <FormattedMessage
              id='ui.txList.txRow.cancel'
              defaultMessage='Cancel'
            />
          </a>
          { isPending
            ? (
              <div>
                <FormattedMessage
                  id='ui.txList.txRow.cancelWarning'
                  defaultMessage='Warning: Editing or Canceling the transaction may not succeed!'
                />
              </div>
            ) : null
          }
        </div>
      );
    }

    let which;

    if (isCancelOpen) {
      which = (
        <FormattedMessage
          id='ui.txList.txRow.verify.cancelEditCancel'
          defaultMessage='Cancel'
        />
      );
    } else {
      which = (
        <FormattedMessage
          id='ui.txList.txRow.verify.cancelEditEdit'
          defaultMessage='Edit'
        />
      );
    }

    return (
      <div className={ styles.pending }>
        <div />
        <div className={ styles.uppercase }>
          <FormattedMessage
            id='ui.txList.txRow.verify'
            defaultMessage='Are you sure?'
          />
        </div>
        <a onClick={ (isCancelOpen) ? this.cancelTx : this.editTx }>
          { which }
        </a>
        <span>{' | '}</span>
        <a onClick={ this.revertEditCancel }>
          <FormattedMessage
            id='ui.txList.txRow.verify.nevermind'
            defaultMessage='Nevermind'
          />
        </a>
      </div>
    );
  }

  getIsKnownContract (address) {
    const { contractAddresses } = this.props;

    return contractAddresses
      .map((a) => a.toLowerCase())
      .includes(address.toLowerCase());
  }

  addressLink (address) {
    const { accountAddresses } = this.props;
    const isAccount = accountAddresses.includes(address);
    const isContract = this.getIsKnownContract(address);

    if (isContract) {
      return `/contracts/${address}`;
    }

    if (isAccount) {
      return `/accounts/${address}`;
    }

    return `/addresses/${address}`;
  }

  getCondition = () => {
    const { blockNumber, tx } = this.props;
    let { time, block = 0 } = tx.condition || {};

    if (time) {
      if ((time.getTime() - Date.now()) >= 0) {
        return (
          <FormattedMessage
            id='ui.txList.txRow.pendingStatus.time'
            defaultMessage='{time} left'
            values={ {
              time: dateDifference(new Date(), time, { compact: true })
            } }
          />
        );
      }
    }

    if (blockNumber) {
      block = blockNumber.minus(block);
      if (block.toNumber() < 0) {
        return (
          <FormattedMessage
            id='ui.txList.txRow.pendingStatus.blocksLeft'
            defaultMessage='{blockNumber} blocks left'
            values={ {
              blockNumber: block.abs().toFormat(0)
            } }
          />
        );
      }
    }

    return 'pending';
  }

  killTx = () => {
    const { killTransaction, tx } = this.props;

    killTransaction(this, tx);
  }

  cancelTx = () => {
    const { cancelTransaction, tx } = this.props;
    const pendingStatus = this.getCondition();
    const isPending = pendingStatus === 'pending';

    if (isPending) {
      this.killTx();
      return;
    }

    cancelTransaction(this, tx);
  }

  editTx = () => {
    const { editTransaction, tx } = this.props;

    editTransaction(this, tx);
  }

  setCancel = () => {
    this.setState({ isCancelOpen: true });
  }

  setEdit = () => {
    this.setState({ isEditOpen: true });
  }

  revertEditCancel = () => {
    this.setState({ isCancelOpen: false, isEditOpen: false });
  }
}

function mapStateToProps (initState) {
  const { accounts, contracts } = initState.personal;
  const accountAddresses = Object.keys(accounts);
  const contractAddresses = Object.keys(contracts);

  return (state) => {
    const { netVersion } = state.nodeStatus;

    return {
      accountAddresses,
      contractAddresses,
      netVersion
    };
  };
}

export default connect(
  mapStateToProps,
  null
)(TxRow);
