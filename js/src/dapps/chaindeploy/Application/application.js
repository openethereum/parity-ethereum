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
        { this.renderContracts(false) }
        { this.renderContracts(true) }
        { this.renderApps() }
        { this.renderContracts(false, true) }
        { this.renderApps(true) }
        { this.renderButtons() }
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
    const { contractBadgereg, contractDappreg, isBadgeDeploying, isContractDeploying, isDappDeploying, haveAllBadges, haveAllContracts, haveAllDapps, registry } = this.store;
    const disableRegistry = registry.address || registry.isDeploying;
    const disableContracts = !registry.address || isContractDeploying || haveAllContracts;
    const disableDapps = !contractDappreg.address || isDappDeploying || haveAllDapps;
    const disableBadges = !registry.address || !contractBadgereg.address || isBadgeDeploying || haveAllBadges;

    return (
      <div className={ styles.buttons }>
        { this.renderButton('registry', this.deployRegistry, disableRegistry) }
        { this.renderButton('contracts', this.deployContracts, disableContracts) }
        { this.renderButton('badges', this.deployBadges, disableBadges) }
        { this.renderButton('apps', this.deployApps, disableDapps) }
      </div>
    );
  }

  renderContracts (isBadges, isExternal) {
    const { badges, contracts, contractBadgereg, registry } = this.store;
    const regaddress = isBadges
      ? contractBadgereg.address
      : registry.address;

    return (
      <div className={ styles.section }>
        <h3>
          {
            isExternal
              ? 'External '
              : ''
          }{
            isBadges
              ? 'Badges '
              : 'Contracts '
          }<small>(registry { regaddress || 'unknown' })</small>
        </h3>
        <div className={ styles.list }>
          {
            isExternal || isBadges
              ? null
              : (
                <Contract
                  contract={ registry }
                  key='registry'
                />
              )
          }
          {
            (isBadges ? badges : contracts)
              .filter((contract) => contract.isExternal === isExternal)
              .map((contract) => {
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

  renderApps (isExternal) {
    const { apps, contractDappreg, contractGithubhint } = this.store;
    const isDisabled = !contractDappreg.isOnChain || !contractGithubhint.isOnChain;

    return (
      <div className={ styles.section }>
        <h3>
          {
            isExternal
              ? 'External '
              : ''
          }Applications <small>(registry {
            contractDappreg.address
              ? contractDappreg.address
              : 'unknown'
          })</small>
        </h3>
        <div className={ styles.list }>
          {
            apps
              .filter((app) => app.isExternal === isExternal)
              .map((app) => {
                return (
                  <Dapp
                    dapp={ app }
                    disabled={ isDisabled }
                    key={ app.id }
                  />
                );
              })
          }
        </div>
      </div>
    );
  }

  deployApps = () => {
    return this.store.deployApps();
  }

  deployBadges = () => {
    return this.store.deployBadges();
  }

  deployContracts = () => {
    return this.store.deployContracts();
  }

  deployRegistry = () => {
    return this.store.deployRegistry();
  }
}
