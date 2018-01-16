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
import { FormattedMessage } from 'react-intl';

import { Button } from '~/ui';
import { CloseIcon, CheckIcon } from '~/ui/Icons';

import Store from './store';
import styles from './extension.css';

@observer
export default class Extension extends Component {
  store = Store.get();

  render () {
    const { showWarning } = this.store;

    if (!showWarning) {
      return null;
    }

    return (
      <div className={ styles.body }>
        <CloseIcon
          className={ styles.close }
          onClick={ this.onClose }
        />
        <p>
          <FormattedMessage
            id='extension.intro'
            defaultMessage='Parity now has an extension available for Chrome that allows safe browsing of Ethereum-enabled decentralized applications. It is highly recommended that you install this extension to further enhance your Parity experience.'
          />
        </p>
        <p className={ styles.buttonrow }>
          <Button
            className={ styles.button }
            icon={ <CheckIcon /> }
            label={
              <FormattedMessage
                id='extension.install'
                defaultMessage='Install the extension now'
              />
            }
            onClick={ this.onInstallClick }
          />
        </p>
      </div>
    );
  }

  onClose = () => {
    this.store.snoozeWarning();
  }

  onInstallClick = () => {
    this.store.installExtension();
  }
}
