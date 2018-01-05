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

import Button from '@parity/ui/lib/Button';
import Portal from '@parity/ui/lib/Portal';
import { DoneIcon } from '@parity/ui/lib/Icons';

import Store from './store';
import TnC from './TnC';

@observer
export default class FirstRun extends Component {
  state = {
    hasAcceptedTnc: false
  }

  store = new Store();

  render () {
    const { hasAcceptedTnc } = this.state;

    if (!this.store.visible) {
      return null;
    }

    return (
      <Portal
        buttons={
          <Button
            disabled={ !hasAcceptedTnc }
            icon={ <DoneIcon /> }
            key='accept'
            label='Close'
            onClick={ this.store.close }
          />
        }
        hideClose
        title={
          <FormattedMessage
            id='firstRun.title.termsOnly'
            defaultMessage='Terms &amp; Conditions'
          />
        }
        open
      >
        <TnC
          hasAccepted={ hasAcceptedTnc }
          onAccept={ this.onAcceptTnC }
        />
      </Portal>
    );
  }

  onAcceptTnC = () => {
    this.setState({
      hasAcceptedTnc: !this.state.hasAcceptedTnc
    });
  }
}
