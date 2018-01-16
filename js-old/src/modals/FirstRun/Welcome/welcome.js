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
import { FormattedMessage } from 'react-intl';

import imagesEthcore from '~/../assets/images/parity-logo-white.svg';

import styles from '../firstRun.css';

const LOGO_STYLE = {
  float: 'right',
  maxWidth: '10em',
  height: 'auto',
  margin: '0 1.5em'
};

export default class FirstRun extends Component {
  render () {
    return (
      <div className={ styles.welcome }>
        <img
          src={ imagesEthcore }
          alt='Parity Ltd.'
          style={ LOGO_STYLE }
        />
        <p>
          <FormattedMessage
            id='firstRun.welcome.greeting'
            defaultMessage='Welcome to Parity, the fastest and simplest way to run your node.'
          />
        </p>
        <p>
          <FormattedMessage
            id='firstRun.welcome.description'
            defaultMessage='As part of a new installation, the next few steps will guide you through the process of setting up your Parity instance and your associated accounts. Our aim is to make it as simple as possible and to get you up and running in record-time, so please bear with us. Once completed you will have -'
          />
        </p>
        <div>
          <ul>
            <li>
              <FormattedMessage
                id='firstRun.welcome.step.privacy'
                defaultMessage='Understood our privacy policy & terms of operation'
              />
            </li>
            <li>
              <FormattedMessage
                id='firstRun.welcome.step.account'
                defaultMessage='Created your first Parity account'
              />
            </li>
            <li>
              <FormattedMessage
                id='firstRun.welcome.step.recovery'
                defaultMessage='Have the ability to recover your account'
              />
            </li>
          </ul>
        </div>
        <p>
          <FormattedMessage
            id='firstRun.welcome.next'
            defaultMessage='Click Next to continue your journey.'
          />
        </p>
      </div>
    );
  }
}
