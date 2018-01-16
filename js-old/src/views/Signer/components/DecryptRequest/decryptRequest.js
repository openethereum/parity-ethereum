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

import Account from '../Account';
import TransactionPendingForm from '../TransactionPendingForm';
import RequestOrigin from '../RequestOrigin';

import styles from '../SignRequest/signRequest.css';

@observer
class DecryptRequest extends Component {
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

  renderDetails () {
    const { api } = this.context;
    const { address, data, netVersion, origin, signerStore } = this.props;
    const { balances, externalLink } = signerStore;

    const balance = balances[address];

    if (!balance) {
      return <div />;
    }

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
        <div className={ styles.info } title={ api.util.sha3(data) }>
          <p>
            <FormattedMessage
              id='signer.decryptRequest.request'
              defaultMessage='A request to decrypt data using your account:'
            />
          </p>

          <div className={ styles.signData }>
            <p>{ data }</p>
          </div>
        </div>
      </div>
    );
  }

  renderActions () {
    const { accounts, address, focus, isFinished, status, data } = this.props;
    const account = accounts[address];

    if (isFinished) {
      if (status === 'confirmed') {
        return (
          <div className={ styles.actions }>
            <span className={ styles.isConfirmed }>
              <FormattedMessage
                id='signer.decryptRequest.state.confirmed'
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
              id='signer.decryptRequest.state.rejected'
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
        focus={ focus }
        isSending={ this.props.isSending }
        netVersion={ this.props.netVersion }
        onConfirm={ this.onConfirm }
        onReject={ this.onReject }
        className={ styles.actions }
        dataToSign={ { decrypt: data } }
      />
    );
  }

  onConfirm = (data) => {
    const { id } = this.props;
    const { password, decrypted, wallet } = data;

    this.props.onConfirm({ id, password, decrypted, wallet });
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
)(DecryptRequest);
