import React, { Component, PropTypes } from 'react';
import formatNumber from 'format-number';

import { ContainerTitle, Input } from '../../../../ui';

import { numberFromString } from './numberFromString';
import { decodeExtraData } from './decodeExtraData';

const toNiceNumber = formatNumber();

export default class MiningSettings extends Component {

  render () {
    const { statusMining, actions } = this.props;

    let onMinGasPriceChange = newVal => {
      actions.modifyMinGasPrice(numberFromString(newVal));
    };

    let onExtraDataChange = (newVal, isResetToDefault) => {
      // In case of resetting to default we are just using raw bytes from defaultExtraData
      // When user sets new value we can safely send a string that will be converted to hex by formatter.
      const val = isResetToDefault ? statusMining.defaultExtraData : newVal;
      actions.modifyExtraData(val);
    };

    let onAuthorChange = newVal => {
      actions.modifyAuthor(newVal);
    };

    let onGasFloorTargetChange = newVal => {
      actions.modifyGasFloorTarget(numberFromString(newVal));
    };

    return (
      <div { ...this._testInherit() }>
        <ContainerTitle title='mining settings' />
        <Input
          label='author'
          hint='the mining author'
          value={ statusMining.author }
          dataSource={ this.props.accounts }
          onChange={ onAuthorChange }
          { ...this._test('author') } />
        <Input
          label='extradata'
          hint='extra data for mined blocks'
          value={ decodeExtraData(statusMining.extraData) }
          onChange={ onExtraDataChange }
          defaultValue={ decodeExtraData(statusMining.defaultExtraData) }
          { ...this._test('extra-data') } />
        <Input
          label='minimal gas price'
          hint='the minimum gas price for mining'
          value={ toNiceNumber(statusMining.minGasPrice) }
          onChange={ onMinGasPriceChange }
          { ...this._test('min-gas-price') } />
        <Input
          label='gas floor target'
          hint='the gas floor target for mining'
          value={ toNiceNumber(statusMining.gasFloorTarget) }
          onChange={ onGasFloorTargetChange }
          { ...this._test('gas-floor-target') } />
      </div>
    );
  }

  static propTypes = {
    accounts: PropTypes.arrayOf(PropTypes.string).isRequired,
    version: PropTypes.string.isRequired,
    statusMining: PropTypes.shape({
      author: PropTypes.string.isRequired,
      extraData: PropTypes.string.isRequired,
      defaultExtraData: PropTypes.string.isRequired,
      minGasPrice: PropTypes.string.isRequired,
      gasFloorTarget: PropTypes.string.isRequired
    }).isRequired,
    actions: PropTypes.shape({
      modifyMinGasPrice: PropTypes.func.isRequired,
      modifyAuthor: PropTypes.func.isRequired,
      modifyGasFloorTarget: PropTypes.func.isRequired,
      modifyExtraData: PropTypes.func.isRequired,
      resetExtraData: PropTypes.func.isRequired
    }).isRequired
  }

}
