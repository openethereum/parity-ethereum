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

import PlaygroundExample from '~/playground/playgroundExample';

import AccountCard from './accountCard';

export default class AccountCardExample extends Component {
  render () {
    const account = {
      address: '0x639ba260535db072a41115c472830846e4e9ad0f',
      description: 'This is a description for the main account',
      meta: {
        tags: [ 'important', 'zargo' ]
      },
      name: 'Main Account'
    };

    const balance = {
      tokens: [
        {
          value: 100000000000000000000,
          token: {
            tag: 'ETH'
          }
        }
      ]
    };

    const accountManyTags = {
      ...account,
      meta: { tags: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => `tag #${n}`) }
    };

    const accountNoTags = {
      ...account,
      meta: { tags: [] }
    };

    return (
      <div>
        <PlaygroundExample name='Standard Account Card'>
          <AccountCard
            account={ account }
            balance={ balance }
          />
        </PlaygroundExample>

        <PlaygroundExample name='Small Account Card'>
          <div style={ { width: 300 } }>
            <AccountCard
              account={ account }
              balance={ balance }
            />
          </div>
        </PlaygroundExample>

        <PlaygroundExample name='Many Tags Account Card'>
          <div style={ { width: 300 } }>
            <AccountCard
              account={ accountManyTags }
              balance={ balance }
            />
          </div>
        </PlaygroundExample>

        <PlaygroundExample name='No Tags Account Card'>
          <div style={ { width: 300 } }>
            <AccountCard
              account={ accountNoTags }
              balance={ balance }
            />
          </div>
        </PlaygroundExample>

        <PlaygroundExample name='Two Account Card'>
          <div style={ { display: 'flex' } }>
            <div style={ { margin: '0 0.5em' } }>
              <AccountCard
                account={ account }
                balance={ balance }
              />
            </div>
            <div style={ { margin: '0 0.5em' } }>
              <AccountCard
                account={ account }
                balance={ balance }
              />
            </div>
          </div>
        </PlaygroundExample>
      </div>
    );
  }
}
