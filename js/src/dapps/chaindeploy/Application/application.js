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
import React, { Component } from 'react';

import Contract from '../Contract';
import Dapp from '../Dapp';
import Store from '../store';

import styles from './application.css';

@observer
export default class Application extends Component {
  store = new Store();

  render () {
    return (
      <div className={ styles.body }>
        { this.renderButtons() }
        { this.renderContracts() }
        { this.renderBuiltins() }
      </div>
    );
  }

  renderButton (text, clickHandler, disabled) {
    const onClick = (event) => {
      if (!disabled) {
        clickHandler(event);
      }
    };

    return (
      <button
        disabled={ disabled }
        onClick={ onClick }
      >
        <div className={ styles.text }>
          { text }
        </div>
      </button>
    );
  }

  renderButtons () {
    const { contractDappreg, isContractDeploying, isDappDeploying, haveAllContracts, haveAllDapps, registry } = this.store;
    const disableRegistry = registry.address || registry.isDeploying;
    const disableContracts = !registry.address || isContractDeploying || haveAllContracts;
    const disableDapps = !contractDappreg.address || isDappDeploying || haveAllDapps;

    return (
      <div className={ styles.buttons }>
        { this.renderButton('registry', this.deployRegistry, disableRegistry) }
        { this.renderButton('contracts', this.deployContracts, disableContracts) }
        { this.renderButton('dapps', this.deployDapps, disableDapps) }
      </div>
    );
  }

  renderContracts () {
    const { contracts, registry } = this.store;

    return (
      <div className={ styles.section }>
        <h3>Contracts</h3>
        <div className={ styles.list }>
          <Contract
            contract={ registry }
            key='registry'
          />
          {
            contracts.map((contract) => {
              return (
                <Contract
                  contract={ contract }
                  disabled={ !registry.address }
                  key={ contract.id }
                />
              );
            })
          }
        </div>
      </div>
    );
  }

  renderBuiltins () {
    const { builtins, contractDappreg, contractGithubhint } = this.store;
    const isDisabled = !contractDappreg.isOnChain || !contractGithubhint.isOnChain;

    return (
      <div className={ styles.section }>
        <h3>Applications</h3>
        <div className={ styles.list }>
          {
            builtins.map((dapp) => {
              return (
                <Dapp
                  dapp={ dapp }
                  disabled={ isDisabled }
                  key={ dapp.id }
                />
              );
            })
          }
        </div>
      </div>
    );
  }

  deployContracts = () => {
    return this.store.deployContracts();
  }

  deployDapps = () => {

  }

  deployRegistry = () => {
    return this.store.deployRegistry();
  }
}
