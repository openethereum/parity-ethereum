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

import React from 'react';
import { observable, action, transaction } from 'mobx';
import { flatMap, uniqBy } from 'lodash';
import { FormattedMessage } from 'react-intl';

import Contracts from '~/contracts';
import { sha3 } from '@parity/api/lib/util/sha3';

const ZERO = /^(0x)?0*$/;

export default class AddressSelectStore {
  @observable loading = false;
  @observable values = [];
  @observable registryValues = [];

  initValues = [];
  regLookups = [
    (query) => {
      query = query.toLowerCase().trim();
      if (query.length === 0 || query === '0x') {
        return null;
      }
      const startsWithQuery = (s) => new RegExp('^' + query, 'i').test(s);

      let address;
      let name = this.reverse[query];

      if (!name) {
        const addr = Object
          .keys(this.reverse)
          .find((addr) => {
            const name = this.reverse[addr];

            return startsWithQuery(addr) || (name && startsWithQuery(name));
          });

        if (addr) {
          address = addr;
          name = this.reverse[addr];
        } else {
          return null;
        }
      }

      return {
        address,
        name,
        description: (
          <FormattedMessage
            id='addressSelect.fromRegistry'
            defaultMessage='{name} (from registry)'
            values={ {
              name
            } }
          />
        )
      };
    }
  ];

  constructor (api) {
    this.api = api;

    const { registry } = Contracts.create(api);

    registry
      .getContract('emailverification')
      .then((emailVerification) => {
        this.regLookups.push((email) => {
          return emailVerification
            .instance
            .reverse
            .call({}, [ sha3.text(email) ])
            .then((address) => {
              return {
                address,
                description: (
                  <FormattedMessage
                    id='addressSelect.fromEmail'
                    defaultMessage='Verified using email {email}'
                    values={ {
                      email
                    } }
                  />
                )
              };
            });
        });
      });

    registry
      .getInstance()
      .then((registryInstance) => {
        this.regLookups.push((name) => {
          return registryInstance
            .getAddress
            .call({}, [ sha3.text(name), 'A' ])
            .then((address) => {
              return {
                address,
                name,
                description: (
                  <FormattedMessage
                    id='addressSelect.fromRegistry'
                    defaultMessage='{name} (from registry)'
                    values={ {
                      name
                    } }
                  />
                )
              };
            });
        });

        this.regLookups.push((address) => {
          return registryInstance
            .reverse
            .call({}, [ address ])
            .then((name) => {
              if (!name) {
                return null;
              }

              return {
                address,
                name,
                description: (
                  <FormattedMessage
                    id='addressSelect.fromRegistry'
                    defaultMessage='{name} (from registry)'
                    values={ {
                      name
                    } }
                  />
                )
              };
            });
        });
      });
  }

  @action setValues (props) {
    const { accounts = {}, contracts = {}, contacts = {}, reverse = {} } = props;

    this.reverse = reverse;

    const accountsN = Object.keys(accounts).length;
    const contractsN = Object.keys(contracts).length;
    const contactsN = Object.keys(contacts).length;

    if (accountsN + contractsN + contactsN === 0) {
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
        values: Object.values(accounts)
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

  @action resetRegistryValues = () => {
    this.registryValues = [];
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

    // Clear the previous results after 50ms
    // if still fetching
    const timeoutId = setTimeout(() => {
      transaction(() => {
        this.registryValues = [];
        this.loading = true;
      });
    }, 50);

    const lookups = this.regLookups.map((regLookup) => regLookup(value));

    // Registries Lookup
    return Promise
      .all(lookups)
      .then((results) => {
        return results
          .filter((result) => result && !ZERO.test(result.address));
      })
      .then((results) => {
        clearTimeout(timeoutId);

        const registryValues = uniqBy(results, (result) => result.address)
          .map((result) => {
            const lowercaseAddress = result.address.toLowerCase();

            const account = flatMap(this.initValues, (cat) => cat.values)
              .find((account) => account.address.toLowerCase() === lowercaseAddress);

            if (account && account.name) {
              result.name = account.name;
            } else if (!result.name) {
              result.name = value;
            }

            return result;
          });

        transaction(() => {
          this.loading = false;
          this.registryValues = registryValues;
        });
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
