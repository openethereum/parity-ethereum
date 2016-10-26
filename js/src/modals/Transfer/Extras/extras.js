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
import { Bar as BarChart } from 'react-chartjs-2';
import Slider from 'material-ui/Slider';
import { isEqual } from 'lodash';
import BigNumber from 'bignumber.js';

import Form, { Input } from '../../../ui/Form';

import styles from '../transfer.css';

export default class Extras extends Component {
  static propTypes = {
    isEth: PropTypes.bool,
    data: PropTypes.string,
    dataError: PropTypes.string,
    gas: PropTypes.string,
    gasEst: PropTypes.string,
    gasError: PropTypes.string,
    gasPrice: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.object
    ]),
    gasPriceDefault: PropTypes.string,
    gasPriceError: PropTypes.string,
    gasPriceStatistics: PropTypes.array,
    total: PropTypes.string,
    totalError: PropTypes.string,
    onChange: PropTypes.func.isRequired
  }

  state = {
    gasPriceChartData: {},
    gasPrice: null,
    gasPriceIndex: 0
  }

  componentWillMount () {
    this.computeGasPriceChart();
    this.setGasPrice();
  }

  componentWillReceiveProps (nextProps) {
    const newGasStats = nextProps
      .gasPriceStatistics
      .map(stat => stat.toNumber())
      .sort();

    const curGasStats = this.props
      .gasPriceStatistics
      .map(stat => stat.toNumber())
      .sort()
      ;

    if (!isEqual(newGasStats, curGasStats)) {
      this.computeGasPriceChart(nextProps);
    }

    if (nextProps.gasPrice !== this.props.gasPrice) {
      this.setGasPrice(nextProps);
    }
  }

  computeGasPriceChart (props = this.props) {
    const { gasPriceStatistics } = props;

    const data = gasPriceStatistics.map(stat => stat.toNumber());

    const gasPriceChartData = {
      labels: data.map((d, index) => index + 1),
      datasets: [{
        label: 'Gas Price',
        type: 'line',
        fill: false,
        borderColor: '#EC932F',
        backgroundColor: '#EC932F',
        pointBorderColor: '#EC932F',
        pointBackgroundColor: '#EC932F',
        pointHoverBackgroundColor: '#EC932F',
        pointHoverBorderColor: '#EC932F',
        yAxisID: 'y-axis-2',
        data
      }, {
        label: 'Gas Price',
        type: 'bar',
        backgroundColor: 'rgba(255, 99, 132, 0.2)',
        borderColor: 'rgba(255,99,132,1)',
        borderWidth: 1,
        yAxisID: 'y-axis-1',
        data
      }]
    };

    this.setState({ gasPriceChartData });
  }

  setGasPrice (props = this.props) {
    const { gasPrice, gasPriceStatistics } = props;

    if (!gasPrice) {
      const index = Math.floor(gasPriceStatistics.length / 2);

      return this.setState({
        gasPrice: gasPriceStatistics[index],
        gasPriceIndex: index
      });
    }

    const bnGasPrice = (typeof gasPrice === 'string')
      ? new BigNumber(gasPrice)
      : gasPrice;

    if (this.state.gasPrice && bnGasPrice.equals(this.state.gasPrice)) {
      return;
    }

    const exactPrices = gasPriceStatistics.filter(p => p.equals(bnGasPrice));

    if (exactPrices.length > 0) {
      const startIndex = gasPriceStatistics.findIndex(p => p.equals(bnGasPrice));
      const index = startIndex + Math.floor(exactPrices.length / 2);

      return this.setState({
        gasPrice: exactPrices[0],
        gasPriceIndex: index
      });
    }

    let index;

    for (index = 0; index < gasPriceStatistics.length - 1; index++) {
      if (gasPriceStatistics[index].greaterThanOrEqualTo(bnGasPrice)) {
        break;
      }
    }

    this.setState({ gasPrice: bnGasPrice, gasPriceIndex: index });
  }

  render () {
    const { gasPrice } = this.state;
    const { gas, gasError, gasEst, gasPriceDefault, gasPriceError, total, totalError } = this.props;

    const gasLabel = `gas amount (estimated: ${gasEst})`;
    const priceLabel = `gas price (current: ${gasPriceDefault})`;

    return (
      <Form>
        <div>
          <p className={ styles.contentTitle }>Gas Price Selection</p>
          <p>
            You can choose the gas price based on the  the octile
            distribution of recent transactions' gas prices.
          </p>
          <p>
            The lower the gas price is, the cheaper the transaction will
            be. The higher the gas price is, the faster it should
            get mined by the network.
          </p>
        </div>

        { this.renderGasPrice() }
        { this.renderGasPriceSlider() }

        <div className={ styles.columns }>
          <div>
            <Input
              label={ gasLabel }
              hint='the amount of gas to use for the transaction'
              error={ gasError }
              value={ gas }
              onChange={ this.onEditGas } />
          </div>
          <div>
            <Input
              label={ priceLabel }
              hint='the price of gas to use for the transaction'
              error={ gasPriceError }
              value={ (gasPrice || '').toString() }
              onChange={ this.onEditGasPrice } />
          </div>
        </div>

        { this.renderData() }

        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label='total transaction amount'
              hint='the total amount of the transaction'
              error={ totalError }
              value={ `${total} ETH` } />
          </div>
        </div>
      </Form>
    );
  }

  renderGasPriceSlider () {
    const { gasPriceIndex } = this.state;
    const { gasPriceStatistics } = this.props;

    return (<div className={ styles.columns }>
      <Slider
        min={ 0 }
        max={ gasPriceStatistics.length - 1 }
        step={ 1 }
        value={ gasPriceIndex }
        onChange={ this.onEditGasPriceSlider }
        style={ {
          flex: 1,
          padding: '0 50px'
        } }
        sliderStyle={ {
          marginBottom: 12
        } }
      />
    </div>);
  }

  renderGasPrice () {
    const { gasPriceChartData } = this.state;

    const chartOptions = {
      maintainAspectRatio: false,
      legend: { display: false },
      tooltips: { callbacks: {
        title: () => '',
        label: (item, data) => {
          const { index } = item;
          const gasPrice = this.props.gasPriceStatistics[index];
          return gasPrice.toFormat(0);
        }
      } },
      scales: {
        xAxes: [{
          display: true,
          gridLines: { display: false },
          labels: { show: true }
        }],
        yAxes: [{
          type: 'linear',
          height: 250,
          display: false,
          position: 'left',
          id: 'y-axis-1',
          gridLines: { display: false },
          labels: { show: true }
        }, {
          type: 'linear',
          height: 250,
          display: false,
          position: 'right',
          id: 'y-axis-2',
          gridLines: { display: false },
          labels: { show: true }
        }]
      }
    };

    return (<div className={ styles.columns }>
      <BarChart
        responsive
        height={ 250 }
        data={ gasPriceChartData }
        options={ chartOptions }
        onElementsClick={ this.onClickGasPrice }
      />
    </div>);
  }

  renderData () {
    const { isEth, data, dataError } = this.props;

    if (!isEth) {
      return null;
    }

    return (
      <div>
        <Input
          hint='the data to pass through with the transaction'
          label='transaction data'
          value={ data }
          error={ dataError }
          onChange={ this.onEditData } />
      </div>
    );
  }

  onClickGasPrice = (items) => {
    const index = items.shift()._index;
    this.onEditGasPriceSlider(null, index);
  }

  onEditGas = (event) => {
    this.props.onChange('gas', event.target.value);
  }

  onEditGasPriceSlider = (event, index) => {
    const { gasPriceStatistics } = this.props;
    const value = gasPriceStatistics[index];

    this.setState({ gasPriceIndex: index });
    this.props.onChange('gasPrice', value);
  }

  onEditGasPrice = (event, value) => {
    this.props.onChange('gasPrice', value);
  }

  onEditData = (event) => {
    this.props.onChange('data', event.target.value);
  }
}
