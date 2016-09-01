import React, { Component, PropTypes } from 'react';

import ContractIcon from 'material-ui/svg-icons/action/code';
import ReactTooltip from 'react-tooltip';

import * as tUtil from '../util/transaction';
import Account from '../Account';
import styles from './TransactionMainDetails.css';

export default class TransactionMainDetails extends Component {

  static propTypes = {
    id: PropTypes.string.isRequired,
    from: PropTypes.string.isRequired,
    fromBalance: PropTypes.object, // eth BigNumber, not required since it might take time to fetch
    value: PropTypes.string.isRequired, // wei hex
    totalValue: PropTypes.object.isRequired, // wei BigNumber
    chain: PropTypes.string.isRequired,
    to: PropTypes.string, // undefined if it's a contract
    toBalance: PropTypes.object, // eth BigNumber - undefined if it's a contract or until it's fetched
    className: PropTypes.string
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
    const { className } = this.props;
    return (
      <div className={ className }>
        { this.renderTransfer() }
        { this.renderContract() }
      </div>
    );
  }

  renderTransfer () {
    const { from, fromBalance, to, toBalance, chain } = this.props;
    if (!to) {
      return;
    }

    return (
      <div className={ styles.transaction }>
        <div className={ styles.from }>
          <Account address={ from } balance={ fromBalance } chain={ chain } />
        </div>
        <div className={ styles.tx }>
          { this.renderValue() }
          <div>&rArr;</div>
          { this.renderTotalValue() }
        </div>
        <div className={ styles.to }>
          <Account address={ to } balance={ toBalance } chain={ chain } />
        </div>
      </div>
    );
  }

  renderContract () {
    const { from, fromBalance, to, chain } = this.props;
    if (to) {
      return;
    }
    return (
      <div className={ styles.transaction }>
        <div className={ styles.from }>
          <Account address={ from } balance={ fromBalance } chain={ chain } />
        </div>
        <div className={ styles.tx }>
          { this.renderValue() }
          <div>&rArr;</div>
          { this.renderTotalValue() }
        </div>
        <div className={ styles.contract }>
          <ContractIcon className={ styles.contractIcon } />
          <br />
          Contract
        </div>
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
