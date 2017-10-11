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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import ReactMarkdown from 'react-markdown';

import { hexToAscii } from '~/api/util/format';
import HardwareStore from '~/mobx/hardwareStore';

import Account from '../Account';
import TransactionPendingForm from '../TransactionPendingForm';
import RequestOrigin from '../RequestOrigin';

import styles from './signRequest.css';

function isAscii (data) {
  for (let i = 2; i < data.length; i += 2) {
    let n = parseInt(data.substr(i, 2), 16);

    if (n < 32 || n >= 128) {
      return false;
    }
  }

  return true;
}

function decodeMarkdown (data) {
  return decodeURIComponent(escape(hexToAscii(data)));
}

export function isMarkdown (data) {
  try {
    const decoded = decodeMarkdown(data);

    for (let i = 0; i < decoded.length; i++) {
      const code = decoded.charCodeAt(i);

      if (code < 32 && code !== 10) {
        return false;
      }
    }

    return decoded.indexOf('#') !== -1 || decoded.indexOf('*') !== -1;
  } catch (error) {
    return false;
  }
}

@observer
class SignRequest extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    data: PropTypes.string.isRequired,
    id: PropTypes.object.isRequired,
    isFinished: PropTypes.bool.isRequired,
    netVersion: PropTypes.string.isRequired,
    signerStore: PropTypes.object.isRequired,

    className: PropTypes.string,
    focus: PropTypes.bool,
    isSending: PropTypes.bool,
    onConfirm: PropTypes.func,
    onReject: PropTypes.func,
    origin: PropTypes.any,
    status: PropTypes.string
  };

  static defaultProps = {
    focus: false,
    origin: {
      type: 'unknown',
      details: ''
    }
  };

  hardwareStore = HardwareStore.get(this.context.api);

  componentWillMount () {
    const { address, signerStore } = this.props;

    signerStore.fetchBalance(address);
  }

  render () {
    const { className } = this.props;

    return (
      <div className={ `${styles.container} ${className}` }>
        { this.renderDetails() }
        { this.renderActions() }
      </div>
    );
  }

  renderData (data) {
    if (isAscii(data)) {
      return hexToAscii(data);
    }

    if (isMarkdown(data)) {
      return (
        <ReactMarkdown source={ decodeMarkdown(data) } />
      );
    }

    return (
      <FormattedMessage
        id='signer.signRequest.unknownBinary'
        defaultMessage='(Unknown binary data)'
      />
    );
  }

  renderDetails () {
    const { address, data, netVersion, origin, signerStore } = this.props;
    const { balances, externalLink } = signerStore;

    const balance = balances[address];

    if (!balance) {
      return <div />;
    }

    const hashToSign = this.context.api.util.sha3(data);

    return (
      <div className={ styles.signDetails }>
        <div className={ styles.address }>
          <Account
            address={ address }
            balance={ balance }
            className={ styles.account }
            externalLink={ externalLink }
            netVersion={ netVersion }
          />
          <RequestOrigin origin={ origin } />
        </div>
        <div className={ styles.info } title={ hashToSign }>
          <p>
            <FormattedMessage
              id='signer.signRequest.request'
              defaultMessage='A request to sign data using your account:'
            />
          </p>
          <div className={ styles.signData }>
            <p>{ this.renderData(data) }</p>
          </div>
          <p>
            <strong>
              <FormattedMessage
                id='signer.signRequest.warning'
                defaultMessage='WARNING: This consequences of doing this may be grave. Confirm the request only if you are sure.'
              />
            </strong>
          </p>
        </div>
      </div>
    );
  }

  renderActions () {
    const { accounts, address, focus, isFinished, status, data } = this.props;
    const account = accounts[address] || {};
    const disabled = account.hardware && !this.hardwareStore.isConnected(address);

    if (isFinished) {
      if (status === 'confirmed') {
        return (
          <div className={ styles.actions }>
            <span className={ styles.isConfirmed }>
              <FormattedMessage
                id='signer.signRequest.state.confirmed'
                defaultMessage='Confirmed'
              />
            </span>
          </div>
        );
      }

      return (
        <div className={ styles.actions }>
          <span className={ styles.isRejected }>
            <FormattedMessage
              id='signer.signRequest.state.rejected'
              defaultMessage='Rejected'
            />
          </span>
        </div>
      );
    }

    return (
      <TransactionPendingForm
        account={ account }
        address={ address }
        disabled={ disabled }
        focus={ focus }
        isSending={ this.props.isSending }
        netVersion={ this.props.netVersion }
        onConfirm={ this.onConfirm }
        onReject={ this.onReject }
        className={ styles.actions }
        dataToSign={ { data } }
      />
    );
  }

  onConfirm = (data) => {
    const { id } = this.props;
    const { password, dataSigned, wallet } = data;

    this.props.onConfirm({ id, password, dataSigned, wallet });
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return {
    accounts
  };
}

export default connect(
  mapStateToProps,
  null
)(SignRequest);
