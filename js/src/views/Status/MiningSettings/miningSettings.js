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

import formatNumber from 'format-number';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { ContainerTitle, Input, TypedInput } from '~/ui';

import { numberFromString } from './numberFromString';
import { decodeExtraData } from './decodeExtraData';

const toNiceNumber = formatNumber();

export default class MiningSettings extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    coinbase: PropTypes.string,
    defaultExtraData: PropTypes.string,
    extraData: PropTypes.string,
    gasFloorTarget: PropTypes.object,
    minGasPrice: PropTypes.object,
    onUpdateSetting: PropTypes.func.isRequired
  };

  render () {
    const { coinbase, defaultExtraData, extraData, gasFloorTarget, minGasPrice } = this.props;
    const decodedExtraData = extraData
      ? decodeExtraData(extraData)
      : '';
    const decodedDefaultExtraData = defaultExtraData
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
        <TypedInput
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
          param='address'
          value={ coinbase }
          onChange={ this.onAuthorChange }
          allowCopy
        />

        <Input
          defaultValue={ decodedDefaultExtraData }
          escape='default'
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
          value={ decodedExtraData }
          onSubmit={ this.onExtraDataChange }
          allowCopy
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
        />
      </div>
    );
  }

  onMinGasPriceChange = (newVal) => {
    const { api } = this.context;

    api.parity
      .setMinGasPrice(numberFromString(newVal))
      .then(() => this.updateMiningSettings());
  };

  onExtraDataChange = (value) => {
    const { api } = this.context;

    api.parity
      .setExtraData(value)
      .then(() => this.updateMiningSettings());
  };

  onAuthorChange = (newVal) => {
    const { api } = this.context;

    api.parity
      .setAuthor(newVal)
      .then(() => this.updateMiningSettings());
  };

  onGasFloorTargetChange = (newVal) => {
    const { api } = this.context;

    api.parity
      .setGasFloorTarget(numberFromString(newVal))
      .then(() => this.updateMiningSettings());
  };

  updateMiningSettings () {
    this.props.onUpdateSetting();
  }
}
