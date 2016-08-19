import React, { Component, PropTypes } from 'react';

import Form, { Input } from '../../../ui/Form';

import styles from '../style.css';

export default class Extras extends Component {
  static propTypes = {
    isEth: PropTypes.bool,
    extraData: PropTypes.string,
    extraDataError: PropTypes.string,
    gas: PropTypes.string,
    gasEst: PropTypes.string,
    gasError: PropTypes.string,
    gasPrice: PropTypes.string,
    gasPriceError: PropTypes.string,
    total: PropTypes.string,
    totalError: PropTypes.string,
    onChange: PropTypes.func.isRequired
  }

  render () {
    const gasLabel = `gas amount (estimated: ${this.props.gasEst})`;

    return (
      <Form>
        { this.renderExtraData() }
        <div className={ styles.columns }>
          <div>
            <Input
              label={ gasLabel }
              hint='the amount of gas to use for the transaction'
              error={ this.props.gasError }
              value={ this.props.gas }
              onChange={ this.onEditGas } />
          </div>
          <div>
            <Input
              label='gas price'
              hint='the price of gas to use for the transaction'
              error={ this.props.gasPriceError }
              value={ this.props.gasPrice }
              onChange={ this.onEditGasPrice } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label='total transaction amount'
              hint='the total amount of the transaction'
              error={ this.props.totalError }
              value={ `${this.props.total} ΞTH` } />
          </div>
        </div>
      </Form>
    );
  }

  renderExtraData () {
    if (!this.props.isEth) {
      return null;
    }

    return (
      <div>
        <Input
          hint='the extraData to pass through with the transaction'
          label='transaction extraData'
          multiLine
          rows={ 1 }
          value={ this.props.extraData }
          onChange={ this.onEditExtraData } />
      </div>
    );
  }

  onEditGas = (event) => {
    this.props.onChange('gas', event.target.value);
  }

  onEditGasPrice = (event) => {
    this.props.onChange('gasPrice', event.target.value);
  }

  onEditExtraData = (event) => {
    this.props.onChange('extraData', event.target.value);
  }
}
