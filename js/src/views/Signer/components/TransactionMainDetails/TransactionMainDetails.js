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

import React, { Component, PropTypes } from 'react';

import ContractIcon from 'material-ui/svg-icons/action/code';
import ReactTooltip from 'react-tooltip';

import * as tUtil from '../util/transaction';
import Account from '../Account';
import styles from './TransactionMainDetails.css';

export default class TransactionMainDetails extends Component {
  static propTypes = {
    id: PropTypes.object.isRequired,
    from: PropTypes.string.isRequired,
    fromBalance: PropTypes.object, // eth BigNumber, not required since it might take time to fetch
    value: PropTypes.object.isRequired, // wei hex
    totalValue: PropTypes.object.isRequired, // wei BigNumber
    isTest: PropTypes.bool.isRequired,
    to: PropTypes.string, // undefined if it's a contract
    toBalance: PropTypes.object, // eth BigNumber - undefined if it's a contract or until it's fetched
    children: PropTypes.node
  };

  componentWillMount () {
    const { value, totalValue } = this.props;

    this.updateDisplayValues(value, totalValue);
  }

  componentWillReceiveProps (nextProps) {
    const { value, totalValue } = nextProps;

    this.updateDisplayValues(value, totalValue);
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

  render () {
    const { to } = this.props;

    return to
      ? this.renderTransfer()
      : this.renderContract();
  }

  renderTransfer () {
    const { children, from, fromBalance, to, toBalance, isTest } = this.props;

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
        <div className={ styles.tx }>
          { this.renderValue() }
          <div>&rArr;</div>
          { this.renderTotalValue() }
        </div>
        <div className={ styles.to }>
          <div className={ styles.account }>
            <Account
              address={ to }
              balance={ toBalance }
              isTest={ isTest } />
          </div>
        </div>
        { children }
      </div>
    );
  }

  renderContract () {
    const { children, from, fromBalance, isTest } = this.props;

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
        <div className={ styles.tx }>
          { this.renderValue() }
          <div>&rArr;</div>
          { this.renderTotalValue() }
        </div>
        <div className={ styles.to }>
          <div className={ styles.account }>
            <ContractIcon className={ styles.contractIcon } />
            <br />
            Contract
          </div>
        </div>
        { children }
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
          data-effect='solid'
          >
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
}
