// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import MapsLocalGasStation from 'material-ui/svg-icons/maps/local-gas-station';
import React, { Component, PropTypes } from 'react';
import ReactTooltip from 'react-tooltip';

import { Button, MethodDecoding } from '~/ui';

import * as tUtil from '../util/transaction';
import Account from '../Account';
import styles from './TransactionMainDetails.css';

export default class TransactionMainDetails extends Component {
  static propTypes = {
    children: PropTypes.node,
    from: PropTypes.string.isRequired,
    fromBalance: PropTypes.object,
    gasStore: PropTypes.object,
    id: PropTypes.object.isRequired,
    isTest: PropTypes.bool.isRequired,
    totalValue: PropTypes.object.isRequired,
    transaction: PropTypes.object.isRequired,
    value: PropTypes.object.isRequired
  };

  componentWillMount () {
    const { value, totalValue } = this.props;

    this.updateDisplayValues(value, totalValue);
  }

  componentWillReceiveProps (nextProps) {
    const { value, totalValue } = nextProps;

    this.updateDisplayValues(value, totalValue);
  }

  render () {
    const { children, from, fromBalance, transaction, isTest } = this.props;

    return (
      <div className={ styles.transaction }>
        <div className={ styles.from }>
          <div className={ styles.account }>
            <Account
              address={ from }
              balance={ fromBalance }
              isTest={ isTest } />
          </div>
        </div>
        <div className={ styles.method }>
          <MethodDecoding
            address={ from }
            historic={ false }
            transaction={ transaction } />
          { this.renderEditGas() }
        </div>
        { children }
      </div>
    );
  }

  renderEditGas () {
    const { gasStore } = this.props;

    if (!gasStore) {
      return null;
    }

    return (
      <div className={ styles.editButtonRow }>
        <Button
          icon={ <MapsLocalGasStation /> }
          label='Edit gas/gasPrice'
          onClick={ this.toggleGasEditor } />
      </div>
    );
  }

  renderValue () {
    const { id } = this.props;
    const { valueDisplay, valueDisplayWei } = this.state;

    return (
      <div>
        <div
          data-tip
          data-for={ 'value' + id }
          data-effect='solid'>
          <strong>{ valueDisplay } </strong>
          <small>ETH</small>
        </div>
        <ReactTooltip id={ 'value' + id }>
          The value of the transaction.<br />
          <strong>{ valueDisplayWei }</strong> <small>WEI</small>
        </ReactTooltip>
      </div>
    );
  }

  renderTotalValue () {
    const { id } = this.props;
    const { totalValueDisplay, totalValueDisplayWei, feeEth } = this.state;

    return (
      <div>
        <div
          data-tip
          data-for={ 'totalValue' + id }
          data-effect='solid'
          data-place='bottom'
          className={ styles.total }>
          { totalValueDisplay } <small>ETH</small>
        </div>
        <ReactTooltip id={ 'totalValue' + id }>
          The value of the transaction including the mining fee is <strong>{ totalValueDisplayWei }</strong> <small>WEI</small>. <br />
          (This includes a mining fee of <strong>{ feeEth }</strong> <small>ETH</small>)
        </ReactTooltip>
      </div>
    );
  }

  updateDisplayValues (value, totalValue) {
    this.setState({
      feeEth: tUtil.calcFeeInEth(totalValue, value),
      valueDisplay: tUtil.getValueDisplay(value),
      valueDisplayWei: tUtil.getValueDisplayWei(value),
      totalValueDisplay: tUtil.getTotalValueDisplay(totalValue),
      totalValueDisplayWei: tUtil.getTotalValueDisplayWei(totalValue)
    });
  }

  toggleGasEditor = () => {
    this.props.gasStore.setEditing(true);
  }
}
