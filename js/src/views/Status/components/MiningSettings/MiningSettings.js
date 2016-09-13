import React, { Component, PropTypes } from 'react';
import formatNumber from 'format-number';

import { ContainerTitle, Input } from '../../../../ui';

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

    return (
      <div { ...this._testInherit() }>
        <ContainerTitle title='mining settings' />
        <Input
          label='author'
          hint='the mining author'
          value={ coinbase }
          onSubmit={ this.onAuthorChange }
          { ...this._test('author') } />
        <Input
          label='extradata'
          hint='extra data for mined blocks'
          value={ decodeExtraData(extraData) }
          onSubmit={ this.onExtraDataChange }
          defaultValue={ decodeExtraData(defaultExtraData) }
          { ...this._test('extra-data') } />
        <Input
          label='minimal gas price'
          hint='the minimum gas price for mining'
          value={ toNiceNumber(minGasPrice) }
          onSubmit={ this.onMinGasPriceChange }
          { ...this._test('min-gas-price') } />
        <Input
          label='gas floor target'
          hint='the gas floor target for mining'
          value={ toNiceNumber(gasFloorTarget) }
          onSubmit={ this.onGasFloorTargetChange }
          { ...this._test('gas-floor-target') } />
      </div>
    );
  }

  onMinGasPriceChange = (newVal) => {
    const { api } = this.context;

    api.ethcore.setMinGasPrice(numberFromString(newVal));
  };

  onExtraDataChange = (newVal, isResetToDefault) => {
    const { api } = this.context;
    const { nodeStatus } = this.props;

    // In case of resetting to default we are just using raw bytes from defaultExtraData
    // When user sets new value we can safely send a string that will be converted to hex by formatter.
    const val = isResetToDefault ? nodeStatus.defaultExtraData : newVal;
    api.ethcore.setExtraData(val);
  };

  onAuthorChange = (newVal) => {
    const { api } = this.context;

    api.ethcore.setAuthor(newVal);
  };

  onGasFloorTargetChange = (newVal) => {
    const { api } = this.context;

    api.ethcore.setGasFloorTarget(numberFromString(newVal));
  };
}
