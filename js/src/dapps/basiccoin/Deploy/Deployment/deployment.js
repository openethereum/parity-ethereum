// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import AddressSelect from '../../AddressSelect';
import Container from '../../Container';
import styles from './deployment.css';

const ERRORS = {
  name: 'specify a valid name >2 & <32 characters',
  tla: 'specify a valid TLA, 3 characters in length',
  usedtla: 'the TLA used is not available for registration',
  supply: 'supply needs to be valid >999 & <1 trillion'
};

export default class Deployment extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired,
    router: PropTypes.object.isRequired,
    managerInstance: PropTypes.object.isRequired,
    registryInstance: PropTypes.object.isRequired,
    tokenregInstance: PropTypes.object.isRequired
  }

  state = {
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
  }

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
            { deployError }
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
    const { accounts } = this.context;
    const { baseText, name, nameError, tla, tlaError, totalSupply, totalSupplyError } = this.state;
    const hasError = !!(nameError || tlaError || totalSupplyError);
    const error = `${styles.input} ${styles.error}`;
    const addresses = Object.keys(accounts);

    // <div className={ styles.input }>
    //   <label>global registration</label>
    //   <select onChange={ this.onChangeRegistrar }>
    //     <option value='no'>No, only for me</option>
    //     <option value='yes'>Yes, for everybody</option>
    //   </select>
    //   <div className={ styles.hint }>
    //     register on network (fee: { globalFeeText }ETH)
    //   </div>
    // </div>

    return (
      <Container>
        <div className={ styles.form }>
          <div className={ styles.input }>
            <label>deployment account</label>
            <AddressSelect
              addresses={ addresses }
              onChange={ this.onChangeFrom }
            />
            <div className={ styles.hint }>
              the owner account to deploy from
            </div>
          </div>
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
              min='1000'
              max='999999999999'
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

  onChangeFrom = (event) => {
    const fromAddress = event.target.value;

    this.setState({ fromAddress });
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
    const totalSupply = parseInt(event.target.value, 10);
    const totalSupplyError = isFinite(totalSupply) && totalSupply > 999
      ? null
      : ERRORS.supply;

    this.setState({ totalSupply, totalSupplyError });
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
    const { base, deployBusy, fromAddress, globalReg, globalFee, name, nameError, tla, tlaError, totalSupply, totalSupplyError } = this.state;
    const hasError = !!(nameError || tlaError || totalSupplyError);

    if (hasError || deployBusy) {
      return;
    }

    const tokenreg = (globalReg ? tokenregInstance : registryInstance).address;
    const values = [base.mul(totalSupply), tla, name, tokenreg];
    const options = {
      from: fromAddress,
      value: globalReg ? globalFee : 0
    };

    this.setState({ deployBusy: true, deployState: 'Estimating gas for the transaction' });

    managerInstance
      .deploy.estimateGas(options, values)
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
        console.error('onDeploy', error);
        this.setState({ deployError: error.message });
      });
  }
}
