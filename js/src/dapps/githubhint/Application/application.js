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

import React, { Component } from 'react';

import { api } from '../parity';
import { attachInterface, subscribeDefaultAddress, unsubscribeDefaultAddress } from '../services';
import Button from '../Button';
import Events from '../Events';
import Loading from '../Loading';

import styles from './application.css';

const INVALID_URL_HASH = '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470';
const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

let nextEventId = 0;

export default class Application extends Component {
  state = {
    defaultAddress: null,
    loading: true,
    url: '',
    urlError: null,
    commit: '',
    commitError: null,
    contentHash: '',
    contentHashError: null,
    contentHashOwner: null,
    registerBusy: false,
    registerError: null,
    registerState: '',
    registerType: 'file',
    repo: '',
    repoError: null,
    subscriptionId: null,
    events: {},
    eventIds: []
  }

  componentDidMount () {
    return Promise
      .all([
        attachInterface(),
        subscribeDefaultAddress((error, defaultAddress) => {
          if (!error) {
            this.setState({ defaultAddress });
          }
        })
      ])
      .then(([state]) => {
        this.setState(Object.assign({}, state, {
          loading: false
        }));
      });
  }

  componentWillUnmount () {
    return unsubscribeDefaultAddress();
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
    const { defaultAddress, registerBusy, url, urlError, contentHash, contentHashError, contentHashOwner, commit, commitError, registerType, repo, repoError } = this.state;

    let hashClass = null;

    if (contentHashError) {
      hashClass = contentHashOwner !== defaultAddress
        ? styles.hashError
        : styles.hashWarning;
    } else if (contentHash) {
      hashClass = styles.hashOk;
    }

    let valueInputs = null;

    if (registerType === 'content') {
      valueInputs = [
        <div className={ styles.capture } key='repo'>
          <input
            type='text'
            placeholder='owner/repo'
            disabled={ registerBusy }
            value={ repo }
            className={ repoError ? styles.error : null }
            onChange={ this.onChangeRepo }
          />
        </div>,
        <div className={ styles.capture } key='hash'>
          <input
            type='text'
            placeholder='commit hash sha3'
            disabled={ registerBusy }
            value={ commit }
            className={ commitError ? styles.error : null }
            onChange={ this.onChangeCommit }
          />
        </div>
      ];
    } else {
      valueInputs = (
        <div className={ styles.capture } key='url'>
          <input
            type='text'
            placeholder='http://domain/filename'
            disabled={ registerBusy }
            value={ url }
            className={ urlError ? styles.error : null }
            onChange={ this.onChangeUrl }
          />
        </div>
      );
    }

    return (
      <div className={ styles.body }>
        <div className={ styles.container }>
          <div className={ styles.form }>
            <div className={ styles.typeButtons }>
              <Button
                disabled={ registerBusy }
                invert={ registerType !== 'file' }
                onClick={ this.onClickTypeNormal }
              >
                File Link
              </Button>
              <Button
                disabled={ registerBusy }
                invert={ registerType !== 'content' }
                onClick={ this.onClickTypeContent }
              >
                Content Bundle
              </Button>
            </div>
            <div className={ styles.box }>
              <div className={ styles.description }>
                Provide a valid URL to register. The content information can be used in other contracts that allows for reverse lookups, e.g. image registries, dapp registries, etc.
              </div>
              { valueInputs }
              <div className={ hashClass }>
                { contentHashError || contentHash }
              </div>
              { registerBusy ? this.renderProgress() : this.renderButtons() }
            </div>
          </div>
        </div>
        <Events
          eventIds={ this.state.eventIds }
          events={ this.state.events }
        />
      </div>
    );
  }

  renderButtons () {
    const { defaultAddress, urlError, repoError, commitError, contentHashError, contentHashOwner } = this.state;

    return (
      <div className={ styles.buttons }>
        <Button
          onClick={ this.onClickRegister }
          disabled={ (contentHashError && contentHashOwner !== defaultAddress) || urlError || repoError || commitError }
        >register url</Button>
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

  onClickTypeNormal = () => {
    const { url } = this.state;

    this.setState({ registerType: 'file', commitError: null, repoError: null }, () => {
      this.onChangeUrl({ target: { value: url } });
    });
  }

  onClickTypeContent = () => {
    const { repo, commit } = this.state;

    this.setState({ registerType: 'content', urlError: null }, () => {
      this.onChangeRepo({ target: { value: repo } });
      this.onChangeCommit({ target: { value: commit } });
    });
  }

  onChangeCommit = (event) => {
    let commit = event.target.value;
    const commitError = null;
    let hasContent = false;

    this.setState({ commit, commitError, contentHashError: null }, () => {
      const { repo } = this.state || '';
      const parts = repo.split('/');

      hasContent = commit.length !== 0 && parts.length === 2 && parts[0].length !== 0 && parts[1].length !== 0;
      if (!commitError && hasContent) {
        this.setState({ contentHashError: 'hash lookup in progress' });
        this.lookupHash(`https://codeload.github.com/${repo}/zip/${commit}`);
      }
    });
  }

  onChangeRepo = (event) => {
    let repo = event.target.value;
    const repoError = null;
    let hasContent = false;

    // TODO: field validation
    if (!repoError) {
      repo = repo.replace('https://github.com/', '');
    }

    this.setState({ repo, repoError, contentHashError: null }, () => {
      const { commit } = this.state || '';
      const parts = repo.split('/');

      hasContent = commit.length !== 0 && parts.length === 2 && parts[0].length !== 0 && parts[1].length !== 0;
      if (!repoError && hasContent) {
        this.setState({ contentHashError: 'hash lookup in progress' });
        this.lookupHash(`https://codeload.github.com/${repo}/zip/${commit}`);
      }
    });
  }

  onChangeUrl = (event) => {
    let url = event.target.value;
    const urlError = null;
    let hasContent = false;

    // TODO: field validation
    if (!urlError) {
      const parts = url.split('/');

      hasContent = parts.length !== 0;

      if (parts[2] === 'github.com' || parts[2] === 'raw.githubusercontent.com') {
        url = `https://raw.githubusercontent.com/${parts.slice(3).join('/')}`.replace('/blob/', '/');
      }
    }

    this.setState({ url, urlError, contentHashError: null }, () => {
      if (!urlError && hasContent) {
        this.setState({ contentHashError: 'hash lookup in progress' });
        this.lookupHash(url);
      }
    });
  }

  onClickRegister = () => {
    const { defaultAddress, commit, commitError, contentHashError, contentHashOwner, url, urlError, registerType, repo, repoError } = this.state;

    // TODO: No errors are currently set, validation to be expanded and added for each
    // field (query is fast to pick up the issues, so not burning atm)
    if ((contentHashError && contentHashOwner !== defaultAddress) || repoError || urlError || commitError) {
      return;
    }

    if (registerType === 'file') {
      this.registerUrl(url);
    } else {
      this.registerContent(repo, commit);
    }
  }

  trackRequest (eventId, promise) {
    return promise
      .then((signerRequestId) => {
        this.setState({
          events: Object.assign({}, this.state.events, {
            [eventId]: Object.assign({}, this.state.events[eventId], {
              signerRequestId,
              registerState: 'Transaction posted, Waiting for transaction authorization'
            })
          })
        });

        return api.pollMethod('parity_checkRequest', signerRequestId);
      })
      .then((txHash) => {
        this.setState({
          events: Object.assign({}, this.state.events, {
            [eventId]: Object.assign({}, this.state.events[eventId], {
              txHash,
              registerState: 'Transaction authorized, Waiting for network confirmations'
            })
          })
        });

        return api.pollMethod('eth_getTransactionReceipt', txHash, (receipt) => {
          if (!receipt || !receipt.blockNumber || receipt.blockNumber.eq(0)) {
            return false;
          }

          return true;
        });
      })
      .then((txReceipt) => {
        this.setState({
          events: Object.assign({}, this.state.events, {
            [eventId]: Object.assign({}, this.state.events[eventId], {
              txReceipt,
              registerBusy: false,
              registerState: 'Network confirmed, Received transaction receipt'
            })
          })
        });
      })
      .catch((error) => {
        console.error('onSend', error);

        this.setState({
          events: Object.assign({}, this.state.events, {
            [eventId]: Object.assign({}, this.state.events[eventId], {
              registerState: error.message,
              registerError: true,
              registerBusy: false
            })
          })
        });
      });
  }

  registerContent (contentRepo, contentCommit) {
    const { defaultAddress, contentHash, instance } = this.state;

    contentCommit = contentCommit.substr(0, 2) === '0x'
      ? contentCommit
      : `0x${contentCommit}`;

    const eventId = nextEventId++;
    const values = [contentHash, contentRepo, contentCommit];
    const options = { from: defaultAddress };

    this.setState({
      eventIds: [eventId].concat(this.state.eventIds),
      events: Object.assign({}, this.state.events, {
        [eventId]: {
          contentHash,
          contentRepo,
          contentCommit,
          defaultAddress,
          registerBusy: true,
          registerState: 'Estimating gas for the transaction',
          timestamp: new Date()
        }
      }),
      url: '',
      commit: '',
      repo: '',
      commitError: null,
      contentHash: '',
      contentHashOwner: null,
      contentHashError: null
    });

    this.trackRequest(
      eventId, instance
        .hint.estimateGas(options, values)
        .then((gas) => {
          this.setState({
            events: Object.assign({}, this.state.events, {
              [eventId]: Object.assign({}, this.state.events[eventId], {
                registerState: 'Gas estimated, Posting transaction to the network'
              })
            })
          });

          const gasPassed = gas.mul(1.2);

          options.gas = gasPassed.toFixed(0);
          console.log(`gas estimated at ${gas.toFormat(0)}, passing ${gasPassed.toFormat(0)}`);

          return instance.hint.postTransaction(options, values);
        })
    );
  }

  registerUrl (contentUrl) {
    const { contentHash, defaultAddress, instance } = this.state;

    const eventId = nextEventId++;
    const values = [contentHash, contentUrl];
    const options = { from: defaultAddress };

    this.setState({
      eventIds: [eventId].concat(this.state.eventIds),
      events: Object.assign({}, this.state.events, {
        [eventId]: {
          contentHash,
          contentUrl,
          defaultAddress,
          registerBusy: true,
          registerState: 'Estimating gas for the transaction',
          timestamp: new Date()
        }
      }),
      url: '',
      commit: '',
      repo: '',
      commitError: null,
      contentHash: '',
      contentHashOwner: null,
      contentHashError: null
    });

    this.trackRequest(
      eventId, instance
        .hintURL.estimateGas(options, values)
        .then((gas) => {
          this.setState({
            events: Object.assign({}, this.state.events, {
              [eventId]: Object.assign({}, this.state.events[eventId], {
                registerState: 'Gas estimated, Posting transaction to the network'
              })
            })
          });

          const gasPassed = gas.mul(1.2);

          options.gas = gasPassed.toFixed(0);
          console.log(`gas estimated at ${gas.toFormat(0)}, passing ${gasPassed.toFormat(0)}`);

          return instance.hintURL.postTransaction(options, values);
        })
    );
  }

  lookupHash (url) {
    const { instance } = this.state;

    if (!url || !url.length) {
      return;
    }

    console.log(`lookupHash ${url}`);

    api.parity
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
