// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import React, { Component } from 'react';

import { api } from '../parity';
import { attachInterface } from '../services';
import Button from '../Button';
import IdentityIcon from '../IdentityIcon';
import Loading from '../Loading';

import styles from './application.css';

const INVALID_URL_HASH = '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470';
const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

export default class Application extends Component {
  state = {
    fromAddress: null,
    loading: true,
    url: '',
    urlError: null,
    contentHash: '',
    contentHashError: null,
    contentHashOwner: null,
    registerBusy: false,
    registerError: null,
    registerState: ''
  }

  componentDidMount () {
    attachInterface()
      .then((state) => {
        this.setState(state, () => {
          this.setState({ loading: false });
        });
      });
  }

  render () {
    const { loading } = this.state;

    return loading
      ? this.renderLoading()
      : this.renderPage();
  }

  renderLoading () {
    return (
      <Loading />
    );
  }

  renderPage () {
    const { fromAddress, registerBusy, url, urlError, contentHash, contentHashError, contentHashOwner } = this.state;

    let hashClass = null;
    if (contentHashError) {
      hashClass = contentHashOwner !== fromAddress ? styles.hashError : styles.hashWarning;
    } else {
      hashClass = styles.hashOk;
    }

    return (
      <div className={ styles.container }>
        <div className={ styles.form }>
          <div className={ styles.box }>
            <div className={ styles.description }>
              Provide a valid URL to register. The content information can be used in other contracts that allows for reverse lookups, e.g. image registries, dapp registries, etc.
            </div>
            <div className={ styles.capture }>
              <input
                type='text'
                placeholder='http://domain/filename'
                disabled={ registerBusy }
                value={ url }
                className={ urlError ? styles.error : null }
                onChange={ this.onChangeUrl } />
            </div>
            <div className={ hashClass }>
              { contentHashError || contentHash }
            </div>
            { registerBusy ? this.renderProgress() : this.renderButtons() }
          </div>
        </div>
      </div>
    );
  }

  renderButtons () {
    const { accounts, fromAddress, url, urlError, contentHashError, contentHashOwner } = this.state;
    const account = accounts[fromAddress];

    return (
      <div className={ styles.buttons }>
        <div className={ styles.addressSelect }>
          <Button invert onClick={ this.onSelectFromAddress }>
            <IdentityIcon address={ account.address } />
            <div>{ account.name || account.address }</div>
          </Button>
        </div>
        <Button
          onClick={ this.onClickRegister }
          disabled={ (!!contentHashError && contentHashOwner !== fromAddress) || !!urlError || url.length === 0 }>register url</Button>
      </div>
    );
  }

  renderProgress () {
    const { registerError, registerState } = this.state;

    if (registerError) {
      return (
        <div className={ styles.progress }>
          <div className={ styles.statusHeader }>
            Your registration has encountered an error
          </div>
          <div className={ styles.statusError }>
            { registerError }
          </div>
        </div>
      );
    }

    return (
      <div className={ styles.progress }>
        <div className={ styles.statusHeader }>
          Your URL is being registered
        </div>
        <div className={ styles.statusState }>
          { registerState }
        </div>
      </div>
    );
  }

  onClickContentHash = () => {
    this.setState({ fileHash: false, commit: '' });
  }

  onClickFileHash = () => {
    this.setState({ fileHash: true, commit: 0 });
  }

  onChangeUrl = (event) => {
    const url = event.target.value;
    let urlError = null;

    if (url && url.length) {
      const re = /^https?:\/\/(?:www\.|(?!www))[^\s\.]+\.[^\s]{2,}/g; // eslint-disable-line
      urlError = re.test(url)
        ? null
        : 'not matching rexex';
    }

    this.setState({ url, urlError, contentHashError: 'hash lookup in progress' }, () => {
      this.lookupHash();
    });
  }

  onClickRegister = () => {
    const { url, urlError, contentHash, contentHashError, contentHashOwner, fromAddress, instance } = this.state;

    if ((!!contentHashError && contentHashOwner !== fromAddress) || !!urlError || url.length === 0) {
      return;
    }

    this.setState({ registerBusy: true, registerState: 'Estimating gas for the transaction' });

    const values = [contentHash, url];
    const options = { from: fromAddress };

    instance
      .hintURL.estimateGas(options, values)
      .then((gas) => {
        this.setState({ registerState: 'Gas estimated, Posting transaction to the network' });

        const gasPassed = gas.mul(1.2);
        options.gas = gasPassed.toFixed(0);
        console.log(`gas estimated at ${gas.toFormat(0)}, passing ${gasPassed.toFormat(0)}`);

        return instance.hintURL.postTransaction(options, values);
      })
      .then((signerRequestId) => {
        this.setState({ signerRequestId, registerState: 'Transaction posted, Waiting for transaction authorization' });

        return api.pollMethod('eth_checkRequest', signerRequestId);
      })
      .then((txHash) => {
        this.setState({ txHash, registerState: 'Transaction authorized, Waiting for network confirmations' });

        return api.pollMethod('eth_getTransactionReceipt', txHash, (receipt) => {
          if (!receipt || !receipt.blockNumber || receipt.blockNumber.eq(0)) {
            return false;
          }

          return true;
        });
      })
      .then((txReceipt) => {
        this.setState({ txReceipt, registerBusy: false, registerState: 'Network confirmed, Received transaction receipt', url: '', contentHash: '' });
      })
      .catch((error) => {
        console.error('onSend', error);
        this.setState({ registerError: error.message });
      });
  }

  onSelectFromAddress = () => {
    const { accounts, fromAddress } = this.state;
    const addresses = Object.keys(accounts);
    let index = 0;

    addresses.forEach((address, _index) => {
      if (address === fromAddress) {
        index = _index;
      }
    });

    index++;
    if (index >= addresses.length) {
      index = 0;
    }

    this.setState({ fromAddress: addresses[index] });
  }

  lookupHash () {
    const { url, instance } = this.state;

    api.ethcore
      .hashContent(url)
      .then((contentHash) => {
        console.log('lookupHash', contentHash);
        if (contentHash === INVALID_URL_HASH) {
          this.setState({ contentHashError: 'invalid url endpoint', contentHash: null });
          return;
        }

        instance.entries
          .call({}, [contentHash])
          .then(([accountSlashRepo, commit, contentHashOwner]) => {
            console.log('lookupHash', accountSlashRepo, api.util.bytesToHex(commit), contentHashOwner);

            if (contentHashOwner !== ZERO_ADDRESS) {
              this.setState({
                contentHashError: contentHash,
                contentHashOwner,
                contentHash
              });
            } else {
              this.setState({ contentHashError: null, contentHashOwner, contentHash });
            }
          });
      })
      .catch((error) => {
        console.error('lookupHash', error);
        this.setState({ contentHashError: error.message, contentHash: null });
      });
  }
}
