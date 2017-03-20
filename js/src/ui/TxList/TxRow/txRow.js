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

import IdentityIcon from '../../IdentityIcon';
import IdentityName from '../../IdentityName';
import MethodDecoding from '../../MethodDecoding';

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
    historic: PropTypes.bool
  };

  static defaultProps = {
    historic: true
  };

  state = {
    isCancelOpen: false,
    isEditOpen: false,
    canceled: false,
    editing: false
  };

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
    const value = api.util.fromWei(_value);

    if (value.eq(0)) {
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
            id='ui.txList.txRow.canceled'
            defaultMessage='CANCELED'
          />
        </div>
      );
    }

    if (editing) {
      return (
        <div className={ styles.pending }>
          <FormattedMessage
            id='ui.txList.txRow.editing'
            defaultMessage='EDITING'
          />
        </div>
      );
    }

    if (!isCancelOpen && !isEditOpen) {
      return (
        <div className={ styles.pending }>
          <span>
            <FormattedMessage
              id='ui.txList.txRow.time'
              defaultMessage='{when}'
              values={ { when: this.getCondition() } }
            />
          </span>
          <div>
            <FormattedMessage
              id='ui.txList.txRow.scheduled'
              defaultMessage='SCHEDULED'
            />
          </div>
          <a onClick={ this.setEdit }>
            <FormattedMessage
              id='ui.txList.txRow.edit'
              defaultMessage='EDIT'
            />
          </a>
          <span>{' | '}</span>
          <a onClick={ this.setCancel }>
            <FormattedMessage
              id='ui.txList.txRow.cancel'
              defaultMessage='CANCEL'
            />
          </a>
        </div>
      );
    }

    return (
      <div className={ styles.pending }>
        <div />
        <div>
          <FormattedMessage
            id='ui.txList.txRow.verify'
            defaultMessage='ARE YOU SURE?'
          />
        </div>
        <a onClick={ (isCancelOpen) ? this.cancelTransaction : this.editTransaction }>
          <FormattedMessage
            id='ui.txList.txRow.verify.cancelEdit'
            defaultMessage='{ which }'
            values={ {
              which: `${(isCancelOpen) ? 'Cancel' : 'Edit'}`
            } }
          />
        </a>
        <span>{' | '}</span>
        <a onClick={ this.nevermind }>
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
    let { time, block } = tx.condition;

    if (time) {
      if ((time.getTime() - Date.now()) >= 0) {
        return `${dateDifference(new Date(), time, { compact: true })} left`;
      } else {
        return 'submitting...';
      }
    } else if (blockNumber) {
      block = blockNumber.minus(block);
      return (block.toNumber() < 0)
        ? block.abs().toFormat(0) + ' blocks left'
        : 'submitting...';
    }
  }

  cancelTransaction = () => {
    const { parity } = this.context.api;
    const { hash } = this.props.tx;

    parity.removeTransaction(hash);
    this.setState({ canceled: true });
  }

  editTransaction = () => {
    const { parity } = this.context.api;
    const { hash, gas, gasPrice, to, from, value, input, condition } = this.props.tx;

    parity.removeTransaction(hash);
    parity.postTransaction({
      from,
      to,
      gas,
      gasPrice,
      value,
      condition,
      data: input
    });
    this.setState({ editing: true });
  }

  setCancel = () => {
    this.setState({ isCancelOpen: true });
  }

  setEdit = () => {
    this.setState({ isEditOpen: true });
  }

  nevermind = () => {
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
