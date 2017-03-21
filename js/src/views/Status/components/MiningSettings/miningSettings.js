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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import formatNumber from 'format-number';

import { ContainerTitle, Input } from '~/ui';

import { numberFromString } from './numberFromString';
import { decodeExtraData } from './decodeExtraData';

const toNiceNumber = formatNumber();

export default class MiningSettings extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    nodeStatus: PropTypes.object
  }

  render () {
    const { nodeStatus } = this.props;
    const { coinbase, defaultExtraData, extraData, gasFloorTarget, minGasPrice } = nodeStatus;

    const extradata = extraData
      ? decodeExtraData(extraData)
      : '';

    const defaultExtradata = defaultExtraData
      ? decodeExtraData(defaultExtraData)
      : '';

    return (
      <div { ...this._testInherit() }>
        <ContainerTitle
          title={
            <FormattedMessage
              id='status.miningSettings.title'
              defaultMessage='mining settings'
            />
          }
        />
        <Input
          label={
            <FormattedMessage
              id='status.miningSettings.input.author.label'
              defaultMessage='author'
            />
          }
          hint={
            <FormattedMessage
              id='status.miningSettings.input.author.hint'
              defaultMessage='the mining author'
            />
          }
          value={ coinbase }
          onSubmit={ this.onAuthorChange }
          allowCopy
          floatCopy
          { ...this._test('author') }
        />

        <Input
          label={
            <FormattedMessage
              id='status.miningSettings.input.extradata.label'
              defaultMessage='extradata'
            />
          }
          hint={
            <FormattedMessage
              id='status.miningSettings.input.extradata.hint'
              defaultMessage='extra data for mined blocks'
            />
          }
          value={ extradata }
          onSubmit={ this.onExtraDataChange }
          defaultValue={ defaultExtradata }
          allowCopy
          floatCopy
          { ...this._test('extra-data') }
        />

        <Input
          label={
            <FormattedMessage
              id='status.miningSettings.input.gasPrice.label'
              defaultMessage='minimal gas price'
            />
          }
          hint={
            <FormattedMessage
              id='status.miningSettings.input.gasPrice.hint'
              defaultMessage='the minimum gas price for mining'
            />
          }
          value={ toNiceNumber(minGasPrice) }
          onSubmit={ this.onMinGasPriceChange }
          allowCopy={ minGasPrice.toString() }
          floatCopy
          { ...this._test('min-gas-price') }
        />

        <Input
          label={
            <FormattedMessage
              id='status.miningSettings.input.gasFloor.label'
              defaultMessage='gas floor target'
            />
          }
          hint={
            <FormattedMessage
              id='status.miningSettings.input.gasFloor.hint'
              defaultMessage='the gas floor target for mining'
            />
          }
          value={ toNiceNumber(gasFloorTarget) }
          onSubmit={ this.onGasFloorTargetChange }
          allowCopy={ gasFloorTarget.toString() }
          floatCopy
          { ...this._test('gas-floor-target') }
        />
      </div>
    );
  }

  onMinGasPriceChange = (newVal) => {
    const { api } = this.context;

    api.parity.setMinGasPrice(numberFromString(newVal));
  };

  onExtraDataChange = (newVal, isResetToDefault) => {
    const { api } = this.context;
    const { nodeStatus } = this.props;

    // In case of resetting to default we are just using raw bytes from defaultExtraData
    // When user sets new value we can safely send a string that will be converted to hex by formatter.
    const val = isResetToDefault ? nodeStatus.defaultExtraData : newVal;

    api.parity.setExtraData(val);
  };

  onAuthorChange = (newVal) => {
    const { api } = this.context;

    api.parity.setAuthor(newVal);
  };

  onGasFloorTargetChange = (newVal) => {
    const { api } = this.context;

    api.parity.setGasFloorTarget(numberFromString(newVal));
  };
}
