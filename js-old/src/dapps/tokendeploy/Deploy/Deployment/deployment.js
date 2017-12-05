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

import React, { Component, PropTypes } from 'react';

import { api } from '../../parity';
import Container from '../../Container';
import styles from './deployment.css';

const DECIMALS = 6;
const BASE = Math.pow(10, DECIMALS);

const ERRORS = {
  name: 'specify a valid name >2 & <32 characters',
  tla: 'specify a valid TLA, 3 characters in length',
  usedtla: 'the TLA used is not available for registration',
  supply: `supply needs to be > 1 & <1 trillion, with no more than ${DECIMALS} decimals`
};

export default class Deployment extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired,
    router: PropTypes.object.isRequired,
    managerInstance: PropTypes.object.isRequired,
    registryInstance: PropTypes.object.isRequired,
    tokenregInstance: PropTypes.object.isRequired
  };

  static initState = {
    base: null,
    deployBusy: false,
    deployDone: false,
    deployError: null,
    deployState: null,
    globalReg: false,
    globalFee: 0,
    globalFeeText: '1.000',
    fromAddress: null,
    name: '',
    nameError: ERRORS.name,
    tla: '',
    tlaError: ERRORS.tla,
    totalSupply: '5000000',
    totalSupplyError: null,
    signerRequestId: null,
    txHash: null
  };

  state = Deployment.initState

  componentDidMount () {
    const { managerInstance, tokenregInstance } = this.context;

    Promise
      .all([
        managerInstance.base.call(),
        tokenregInstance.fee.call()
      ])
      .then(([base, globalFee]) => {
        this.setState({
          base,
          baseText: base.toFormat(0),
          globalFee,
          globalFeeText: api.util.fromWei(globalFee).toFormat(3)
        });
      });
  }

  reset () {
    this.setState(Deployment.initState, () => this.componentDidMount());
  }

  render () {
    const { deployBusy } = this.state;

    return deployBusy
      ? this.renderDeploying()
      : this.renderForm();
  }

  renderDeploying () {
    const { deployDone, deployError, deployState } = this.state;

    if (deployDone) {
      return (
        <Container>
          <div className={ styles.statusHeader }>
            Your token has been deployed
          </div>
        </Container>
      );
    }

    if (deployError) {
      return (
        <Container>
          <div className={ styles.statusHeader }>
            Your deployment has encountered an error
          </div>
          <div className={ styles.statusError }>
            { deployError.message }
          </div>
        </Container>
      );
    }

    return (
      <Container>
        <div className={ styles.statusHeader }>
          Your token is currently being deployed to the network
        </div>
        <div className={ styles.statusState }>
          { deployState }
        </div>
      </Container>
    );
  }

  renderForm () {
    const { baseText, name, nameError, tla, tlaError, totalSupply, totalSupplyError } = this.state;
    const hasError = !!(nameError || tlaError || totalSupplyError);
    const error = `${styles.input} ${styles.error}`;

    return (
      <Container>
        <div className={ styles.form }>
          <div className={ nameError ? error : styles.input }>
            <label>token name</label>
            <input
              value={ name }
              name='name'
              onChange={ this.onChangeName }
            />
            <div className={ styles.hint }>
              { nameError || 'an identifying name for the token' }
            </div>
          </div>
          <div className={ tlaError ? error : styles.input }>
            <label>token TLA</label>
            <input
              className={ styles.small }
              name='tla'
              value={ tla }
              onChange={ this.onChangeTla }
            />
            <div className={ styles.hint }>
              { tlaError || 'unique network acronym for this token' }
            </div>
          </div>
          <div className={ totalSupplyError ? error : styles.input }>
            <label>token supply</label>
            <input
              type='number'
              step={ 1 }
              min={ 1 }
              max='999999999999999'
              name='totalSupply'
              value={ totalSupply }
              onChange={ this.onChangeSupply }
            />
            <div className={ styles.hint }>
              { totalSupplyError || `number of tokens (base: ${baseText})` }
            </div>
          </div>
          <div className={ styles.input }>
            <label />
            <div className={ styles.buttonRow }>
              <div
                className={ styles.button }
                disabled={ hasError }
                onClick={ this.onDeploy }
              >
                Deploy Token
              </div>
            </div>
          </div>
        </div>
      </Container>
    );
  }

  onChangeName = (event) => {
    const name = event.target.value;
    const nameError = name && (name.length > 2) && (name.length < 32)
      ? null
      : ERRORS.name;

    this.setState({ name, nameError });
  }

  onChangeRegistrar = (event) => {
    this.setState({ globalReg: event.target.value === 'yes' }, this.testTlaAvailability);
  }

  onChangeSupply = (event) => {
    const { value } = event.target;
    const floatValue = parseFloat(value, 10);
    const convertedTotalSupply = floatValue * BASE;
    const totalSupplyError = Number.isInteger(convertedTotalSupply) && floatValue >= 1
      ? null
      : ERRORS.supply;

    this.setState({ totalSupply: value, totalSupplyError });
  }

  onChangeTla = (event) => {
    const _tla = event.target.value;
    const tla = _tla && (_tla.length > 3)
      ? _tla.substr(0, 3)
      : _tla;
    const tlaError = tla && (tla.length === 3)
      ? null
      : ERRORS.tla;

    this.setState({ tla, tlaError }, this.testTlaAvailability);
  }

  testTlaAvailability = () => {
    const { registryInstance, tokenregInstance } = this.context;
    const { globalReg, tla, tlaError } = this.state;
    const tokenreg = globalReg ? tokenregInstance : registryInstance;

    if (tlaError && tlaError !== ERRORS.usedtla) {
      return;
    }

    tokenreg
      .fromTLA.call({}, [tla])
      .then(([id, addr, base, name, owner]) => {
        if (owner !== '0x0000000000000000000000000000000000000000') {
          this.setState({ tlaError: ERRORS.usedtla });
        } else if (tlaError === ERRORS.usedtla) {
          this.setState({ tlaError: null });
        }
      })
      .catch((error) => {
        console.log('testTlaAvailability', error);
      });
  }

  onDeploy = () => {
    const { managerInstance, registryInstance, tokenregInstance } = this.context;
    const { base, deployBusy, globalReg, name, nameError, tla, tlaError, totalSupply, totalSupplyError } = this.state;
    const hasError = !!(nameError || tlaError || totalSupplyError);

    if (hasError || deployBusy) {
      return;
    }

    const registry = globalReg ? tokenregInstance : registryInstance;
    const tokenreg = registry.address;

    const values = [base.mul(totalSupply), tla, name, tokenreg];
    const options = {};

    this.setState({ deployBusy: true, deployState: 'Estimating gas for the transaction' });

    return registry.fee.call({}, [])
      .then((fee) => {
        console.log('deploying with fee of', fee.toFixed());
        options.value = fee;

        return api.parity.defaultAccount();
      })
      .then((defaultAddress) => {
        options.from = defaultAddress;

        return managerInstance.deploy.estimateGas(options, values);
      })
      .then((gas) => {
        this.setState({ deployState: 'Gas estimated, Posting transaction to the network' });

        const gasPassed = gas.mul(1.2);

        options.gas = gasPassed.toFixed(0);
        console.log(`gas estimated at ${gas.toFormat(0)}, passing ${gasPassed.toFormat(0)}`);

        return managerInstance.deploy.postTransaction(options, values);
      })
      .then((signerRequestId) => {
        this.setState({ signerRequestId, deployState: 'Transaction posted, Waiting for transaction authorization' });

        return api.pollMethod('parity_checkRequest', signerRequestId);
      })
      .then((txHash) => {
        this.setState({ txHash, deployState: 'Transaction authorized, Waiting for network confirmations' });

        return api.pollMethod('eth_getTransactionReceipt', txHash, (receipt) => {
          if (!receipt || !receipt.blockNumber || receipt.blockNumber.eq(0)) {
            return false;
          }

          return true;
        });
      })
      .then((txReceipt) => {
        this.setState({ txReceipt, deployDone: true, deployState: 'Network confirmed, Received transaction receipt' });
      })
      .catch((error) => {
        if (error.type === 'REQUEST_REJECTED') {
          return this.reset();
        }

        console.error('onDeploy', error);
        this.setState({ deployError: error });
      });
  }
}
