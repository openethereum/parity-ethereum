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

import React from 'react';
import { observable, action } from 'mobx';
import { flatMap } from 'lodash';
import { FormattedMessage } from 'react-intl';

import Contracts from '~/contracts';
import { sha3 } from '~/api/util/sha3';

export default class AddressSelectStore {

  @observable values = [];
  @observable registryValues = [];

  initValues = [];
  regLookups = [];

  constructor (api) {
    this.api = api;

    const { registry } = Contracts.create(api);

    registry
      .getContract('emailverification')
      .then((emailVerification) => {
        this.regLookups.push({
          lookup: (value) => {
            return emailVerification
              .instance
              .reverse.call({}, [ sha3(value) ]);
          },
          describe: (value) => (
            <FormattedMessage
              id='addressSelect.fromEmail'
              defaultMessage='Verified using email {value}'
              values={ {
                value
              } }
            />
          )
        });
      });

    registry
      .getInstance()
      .then((registryInstance) => {
        this.regLookups.push({
          lookup: (value) => {
            return registryInstance
              .getAddress.call({}, [ sha3(value), 'A' ]);
          },
          describe: (value) => (
            <FormattedMessage
              id='addressSelect.fromRegistry'
              defaultMessage='{value} (from registry)'
              values={ {
                value
              } }
            />
          )
        });
      });
  }

  @action setValues (props) {
    const { accounts = {}, contracts = {}, contacts = {}, wallets = {} } = props;

    const accountsN = Object.keys(accounts).length;
    const contractsN = Object.keys(contracts).length;
    const contactsN = Object.keys(contacts).length;
    const walletsN = Object.keys(wallets).length;

    if (accountsN + contractsN + contactsN + walletsN === 0) {
      return;
    }

    this.initValues = [
      {
        key: 'accounts',
        label: (
          <FormattedMessage
            id='addressSelect.labels.accounts'
            defaultMessage='accounts'
          />
        ),
        values: [].concat(
          Object.values(wallets),
          Object.values(accounts)
        )
      },
      {
        key: 'contacts',
        label: (
          <FormattedMessage
            id='addressSelect.labels.contacts'
            defaultMessage='contacts'
          />
        ),
        values: Object.values(contacts)
      },
      {
        key: 'contracts',
        label: (
          <FormattedMessage
            id='addressSelect.labels.contracts'
            defaultMessage='contracts'
          />
        ),
        values: Object.values(contracts)
      }
    ].filter((cat) => cat.values.length > 0);

    this.handleChange();
  }

  @action handleChange = (value = '') => {
    let index = 0;

    this.values = this.initValues
      .map((category) => {
        const filteredValues = this
          .filterValues(category.values, value)
          .map((value) => {
            index++;

            return {
              index: parseInt(index),
              ...value
            };
          });

        return {
          label: category.label,
          values: filteredValues
        };
      });

    // Registries Lookup
    this.registryValues = [];

    const lookups = this.regLookups.map((regLookup) => regLookup.lookup(value));

    Promise
      .all(lookups)
      .then((results) => {
        return results
          .map((result, index) => {
            if (/^(0x)?0*$/.test(result)) {
              return;
            }

            const lowercaseResult = result.toLowerCase();

            const account = flatMap(this.initValues, (cat) => cat.values)
              .find((account) => account.address.toLowerCase() === lowercaseResult);

            return {
              description: this.regLookups[index].describe(value),
              address: result,
              name: account && account.name || value
            };
          })
          .filter((data) => data);
      })
      .then((registryValues) => {
        this.registryValues = registryValues;
      });
  }

  /**
   * Filter the given values based on the given
   * filter
   */
  filterValues = (values = [], _filter = '') => {
    const filter = _filter.toLowerCase();

    return values
      // Remove empty accounts
      .filter((a) => a)
      .filter((account) => {
        const address = account.address.toLowerCase();
        const inAddress = address.includes(filter);

        if (!account.name || inAddress) {
          return inAddress;
        }

        const name = account.name.toLowerCase();
        const inName = name.includes(filter);
        const { meta = {} } = account;

        if (!meta.tags || inName) {
          return inName;
        }

        const tags = (meta.tags || []).join('');
        return tags.includes(filter);
      })
      .sort((accA, accB) => {
        const nameA = accA.name || accA.address;
        const nameB = accB.name || accB.address;

        return nameA.localeCompare(nameB);
      });
  }

}
