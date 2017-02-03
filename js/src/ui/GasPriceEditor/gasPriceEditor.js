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

import BigNumber from 'bignumber.js';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Input, InputDate, InputTime, RadioButtons } from '../Form';
import GasPriceSelector from '../GasPriceSelector';

import Store, { CONDITIONS } from './store';
import styles from './gasPriceEditor.css';

const CONDITION_VALUES = [
  {
    label: (
      <FormattedMessage
        id='txEditor.condition.none'
        defaultMessage='No conditions'
      />
    ),
    key: CONDITIONS.NONE
  },
  {
    label: (
      <FormattedMessage
        id='txEditor.condition.blocknumber'
        defaultMessage='Send after BlockNumber'
      />
    ),
    key: CONDITIONS.BLOCK
  },
  {
    label: (
      <FormattedMessage
        id='txEditor.condition.datetime'
        defaultMessage='Send after Date & Time'
      />
    ),
    key: CONDITIONS.TIME
  }
];

@observer
export default class GasPriceEditor extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    children: PropTypes.node,
    onChange: PropTypes.func,
    store: PropTypes.object.isRequired
  }

  static Store = Store;

  render () {
    const { api } = this.context;
    const { children, store } = this.props;
    const { conditionType, errorGas, errorPrice, errorTotal, estimated, gas, histogram, price, priceDefault, totalValue } = store;

    const eth = api.util.fromWei(totalValue).toFormat();
    const gasLabel = `gas (estimated: ${new BigNumber(estimated).toFormat()})`;
    const priceLabel = `price (current: ${new BigNumber(priceDefault).toFormat()})`;

    return (
      <div className={ styles.container }>
        <RadioButtons
          className={ styles.conditionRadio }
          label={
            <FormattedMessage
              id='txEditor.condition.label'
              defaultMessage='Condition where transaction activates'
            />
          }
          onChange={ this.onChangeConditionType }
          value={ conditionType }
          values={ CONDITION_VALUES }
        />
        { this.renderConditions() }

        <div className={ styles.graphContainer }>
          <div className={ styles.graphColumn }>
            <GasPriceSelector
              histogram={ histogram }
              onChange={ this.onEditGasPrice }
              price={ price }
            />
            <div className={ styles.gasPriceDesc }>
              <FormattedMessage
                id='txEditor.gas.info'
                defaultMessage='You can choose the gas price based on the distribution of recent included transaction gas prices. The lower the gas price is, the cheaper the transaction will be. The higher the gas price is, the faster it should get mined by the network.'
              />
            </div>
          </div>

          <div className={ styles.editColumn }>
            <div className={ styles.row }>
              <Input
                error={ errorGas }
                hint='the amount of gas to use for the transaction'
                label={ gasLabel }
                min={ 1 }
                onChange={ this.onEditGas }
                type='number'
                value={ gas }
              />
              <Input
                error={ errorPrice }
                hint='the price of gas to use for the transaction'
                label={ priceLabel }
                min={ 1 }
                onChange={ this.onEditGasPrice }
                type='number'
                value={ price }
              />
            </div>
            <div className={ styles.row }>
              <Input
                disabled
                error={ errorTotal }
                hint='the total amount of the transaction'
                label='total transaction amount'
                value={ `${eth} ETH` }
              />
            </div>
            <div className={ styles.row }>
              { children }
            </div>
          </div>
        </div>
      </div>
    );
  }

  renderConditions () {
    const { conditionType, condition, conditionBlockError } = this.props.store;

    if (conditionType === CONDITIONS.NONE) {
      return null;
    }

    if (conditionType === CONDITIONS.BLOCK) {
      return (
        <div className={ styles.conditionContainer }>
          <div className={ styles.input }>
            <Input
              error={ conditionBlockError }
              hint={
                <FormattedMessage
                  id='txEditor.condition.block.hint'
                  defaultMessage='The minimum block to send from'
                />
              }
              label={
                <FormattedMessage
                  id='txEditor.condition.block.label'
                  defaultMessage='Transaction send block'
                />
              }
              min={ 1 }
              onChange={ this.onChangeConditionBlock }
              type='number'
              value={ condition.block }
            />
          </div>
        </div>
      );
    }

    return (
      <div className={ styles.conditionContainer }>
        <div className={ styles.input }>
          <InputDate
            hint={
              <FormattedMessage
                id='txEditor.condition.date.hint'
                defaultMessage='The minimum date to send from'
              />
            }
            label={
              <FormattedMessage
                id='txEditor.condition.date.label'
                defaultMessage='Transaction send date'
              />
            }
            onChange={ this.onChangeConditionDateTime }
            value={ condition.time }
          />
        </div>
        <div className={ styles.input }>
          <InputTime
            hint={
              <FormattedMessage
                id='txEditor.condition.time.hint'
                defaultMessage='The minimum time to send from'
              />
            }
            label={
              <FormattedMessage
                id='txEditor.condition.time.label'
                defaultMessage='Transaction send time'
              />
            }
            onChange={ this.onChangeConditionDateTime }
            value={ condition.time }
          />
        </div>
      </div>
    );
  }

  onEditGas = (event, gas) => {
    const { store, onChange } = this.props;

    store.setGas(gas);
    onChange && onChange('gas', gas);
  }

  onEditGasPrice = (event, price) => {
    const { store, onChange } = this.props;

    store.setPrice(price);
    onChange && onChange('gasPrice', price);
  }

  onChangeConditionType = (conditionType) => {
    this.props.store.setConditionType(conditionType.key);
  }

  onChangeConditionBlock = (event, blockNumber) => {
    this.props.store.setConditionBlockNumber(blockNumber);
  }

  onChangeConditionDateTime = (event, datetime) => {
    this.props.store.setConditionDateTime(datetime);
  }
}
