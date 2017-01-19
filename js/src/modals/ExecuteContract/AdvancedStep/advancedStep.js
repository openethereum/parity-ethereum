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
import { FormattedMessage } from 'react-intl';

import { Input, GasPriceEditor } from '~/ui';

import styles from '../executeContract.css';

export default class AdvancedStep extends Component {
  static propTypes = {
    gasStore: PropTypes.object.isRequired,
    minBlock: PropTypes.string,
    minBlockError: PropTypes.string,
    onMinBlockChange: PropTypes.func
  };

  render () {
    const { gasStore, minBlock, minBlockError, onMinBlockChange } = this.props;

    return (
      <div>
        <Input
          error={ minBlockError }
          hint={
            <FormattedMessage
              id='executeContract.advanced.minBlock.hint'
              defaultMessage='Only post the transaction after this block'
            />
          }
          label={
            <FormattedMessage
              id='executeContract.advanced.minBlock.label'
              defaultMessage='BlockNumber to send from'
            />
          }
          value={ minBlock }
          onSubmit={ onMinBlockChange }
        />
        <div className={ styles.gaseditor }>
          <GasPriceEditor store={ gasStore } />
        </div>
      </div>
    );
  }
}
