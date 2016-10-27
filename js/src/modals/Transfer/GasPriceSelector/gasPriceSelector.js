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
import { Bar as BarChart, Line as LineChart, Bubble as BubbleChart } from 'react-chartjs-2';
import Slider from 'material-ui/Slider';
import BigNumber from 'bignumber.js';

import componentStyles from './gasPriceSelector.css';
import mainStyles from '../transfer.css';

const styles = Object.assign({}, mainStyles, componentStyles);

const BAR_CHART_OPTIONS = {
  animation: false,
  maintainAspectRatio: false,
  legend: { display: false },
  tooltips: { callbacks: { title: () => '' } },
  scales: {
    xAxes: [{
      display: true,
      gridLines: { display: false },
      labels: { show: true },
      barPercentage: 1.0,
      categoryPercentage: 1.0
    }],
    yAxes: [{ display: false, type: 'linear' }]
  }
};

const LINE_CHART_OPTIONS = {
  animation: false,
  maintainAspectRatio: false,
  legend: { display: false },
  scales: {
    xAxes: [{ display: true, ticks: { display: true } }],
    yAxes: [{ display: false, type: 'linear' }]
  }
};

const BAR_CHART_DATA = {
  datasets: [{
    label: 'Gas Price',
    type: 'bar',
    backgroundColor: 'rgba(255, 99, 132, 0.2)',
    borderColor: 'rgba(255,99,132,1)',
    borderWidth: 1
  }]
};

const LINE_CHART_DATA = {
  datasets: [{
    label: 'Gas Price',
    type: 'line',
    cubicInterpolationMode: 'monotone',
    lineTension: 0,
    fill: false,
    borderColor: '#EC932F',
    backgroundColor: '#EC932F',
    pointRadius: 0
  }]
};

const BUBBLE_CHART_OPTIONS = {
  animation: false,
  maintainAspectRatio: false,
  legend: { display: false },
  scales: {
    xAxes: [{ display: true, ticks: {
      min: 0,
      callback: () => '  '
    } }],
    yAxes: [{ display: false }]
  }
};

const BUBBLE_CHART_DATA = {
  datasets: [{
    label: 'Gas Point',
    backgroundColor: '#EC932F',
    hoverBackgroundColor: '#EC932F',
    data: [{
      x: 0, y: 0, r: 0
    }]
  }]
};

const CHART_TICKS = {
  min: 0,
  beginAtZero: true
};

export default class GasPriceSelector extends Component {
  static propTypes = {
    gasPriceStatistics: PropTypes.array.isRequired,
    onChange: PropTypes.func.isRequired,

    gasPrice: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.object
    ])
  }

  state = {
    gasPrice: null,
    sliderValue: 0,

    lineChartData: null,
    barChartData: null,
    lineChartOptions: null,
    barChartOptions: null,
    bubbleChartData: null,
    bubbleChartOptions: null
  }

  componentWillMount () {
    this.computeCharts();
    this.setGasPrice();
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.gasPrice !== this.props.gasPrice) {
      this.setGasPrice(nextProps);
    }
  }

  componentWillUpdate (nextProps, nextState) {
    if (Math.floor(nextState.sliderValue) !== Math.floor(this.state.sliderValue)) {
      this.updateSelectedBarChart(nextState);
    }
  }

  render () {
    return (
      <div>
        { this.renderChart() }
        { this.renderSlider() }
      </div>
    );
  }

  renderSlider () {
    const { sliderValue } = this.state;
    const { gasPriceStatistics } = this.props;

    return (<div className={ styles.columns }>
      <Slider
        min={ 0 }
        max={ gasPriceStatistics.length - 1 }
        value={ sliderValue }
        onChange={ this.onEditGasPriceSlider }
        style={ {
          flex: 1,
          padding: '0 0.3em'
        } }
        sliderStyle={ {
          marginBottom: 12
        } }
      />
    </div>);
  }

  renderChart () {
    const {
      lineChartData, lineChartOptions,
      barChartData, barChartOptions,
      bubbleChartData, bubbleChartOptions
    } = this.state;

    if (!lineChartData) {
      return null;
    }

    const height = 350;

    return (<div className={ styles.columns }>
      <div style={ { flex: 1, height } }>
        <div className={ styles.chart }>
          <LineChart
            responsive
            height={ height }
            data={ lineChartData }
            options={ lineChartOptions }
          />
        </div>
        <div className={ styles.chart }>
          <BubbleChart
            ref='bubbleChart'
            responsive
            height={ height }
            data={ bubbleChartData }
            options={ bubbleChartOptions }
          />
        </div>
        <div className={ styles.chart }>
          <BarChart
            ref='barChart'
            responsive
            height={ height }
            data={ barChartData }
            options={ barChartOptions }
            onElementsClick={ this.onClickGasPrice }
          />
        </div>
      </div>
    </div>);
  }

  computeChartsData () {
    const { gasPriceChartData } = this.state;
    const [ avgData, data ] = gasPriceChartData;

    const lineChartData = { ...LINE_CHART_DATA };
    const lineChartOptions = { ...LINE_CHART_OPTIONS };
    const barChartData = { ...BAR_CHART_DATA };
    const barChartOptions = { ...BAR_CHART_OPTIONS };
    const bubbleChartData = { ...BUBBLE_CHART_DATA };
    const bubbleChartOptions = { ...BUBBLE_CHART_OPTIONS };

    const ticks = {
      ...CHART_TICKS,
      max: data[data.length - 1] * 1.1
    };

    barChartOptions.tooltips.callbacks.label = (item, data) => {
      const { index } = item;
      const gasPriceA = this.props.gasPriceStatistics[index];
      const gasPriceB = this.props.gasPriceStatistics[index + 1];
      return `${gasPriceA.toFormat(0)} => ${gasPriceB.toFormat(0)}`;
    };

    barChartOptions.scales.yAxes[0].ticks = ticks;
    lineChartOptions.scales.yAxes[0].ticks = ticks;

    bubbleChartOptions.scales.yAxes[0].ticks = ticks;
    bubbleChartOptions.scales.xAxes[0].ticks.max = avgData.length - 0.05;

    lineChartData.labels = data.map((d, index) => ' ');
    lineChartData.datasets[0].data = data;

    barChartData.labels = avgData.map((d, index) => index + 1);
    barChartData.datasets[0].data = avgData;
    barChartData.datasets[0].backgroundColor = this.getBarChartBackgrounds();

    this.setState({
      lineChartData, barChartData, bubbleChartData,
      lineChartOptions, barChartOptions, bubbleChartOptions
    });
  }

  computeCharts (props = this.props) {
    const { gasPriceStatistics } = props;

    const data = gasPriceStatistics.map(stat => stat.toNumber());
    const avgData = gasPriceStatistics
      .slice(0, -1)
      .map((a, index) => {
        const b = gasPriceStatistics[index + 1];
        return a.plus(b).dividedBy(2).toNumber();
      });

    this.setState(
      { gasPriceChartData: [ avgData, data ] },
      () => this.computeChartsData()
    );
  }

  updateSelectedBarChart (state = this.state) {
    const barChartData = { ...state.barChartData };
    barChartData.datasets[0].backgroundColor = this.getBarChartBackgrounds(state);

    this.setState({ barChartData }, () => {
      this.refs.barChart.chart_instance.update();
    });
  }

  getBarChartBackgrounds (state = this.state) {
    const { sliderValue, gasPriceChartData } = state;
    const [ avgData ] = gasPriceChartData;

    const index = Math.min(
      avgData.length - 1,
      Math.floor(sliderValue)
    );

    const backgrounds = avgData
      .map((d, idx) => (idx === index) || (index === sliderValue && idx === index - 1) ? 0.4 : 0.2)
      .map((a) => `rgba(255, 99, 132, ${a})`);

    return backgrounds;
  }

  setGasPrice (props = this.props) {
    const { gasPrice, gasPriceStatistics } = props;

    // If no gas price yet...
    if (!gasPrice) {
      const index = Math.floor(gasPriceStatistics.length / 2);
      return this.setSliderValue(index + 0.5);
    }

    const bnGasPrice = (typeof gasPrice === 'string')
      ? new BigNumber(gasPrice)
      : gasPrice;

    // If gas price hasn't changed
    if (this.state.gasPrice && bnGasPrice.equals(this.state.gasPrice)) {
      return;
    }

    const exactPrices = gasPriceStatistics.filter(p => p.equals(bnGasPrice));

    if (exactPrices.length > 0) {
      const startIndex = gasPriceStatistics.findIndex(p => p.equals(bnGasPrice));
      const sliderValue = startIndex + Math.floor(exactPrices.length / 2);

      return this.setSliderValue(sliderValue, bnGasPrice);
    }

    let minIndex = -1;

    while (minIndex < gasPriceStatistics.length - 1) {
      if (bnGasPrice.lessThanOrEqualTo(gasPriceStatistics[minIndex + 1])) {
        break;
      }

      minIndex++;
    }

    if (minIndex < 0) {
      return this.setSliderValue(0, bnGasPrice);
    }

    if (minIndex >= gasPriceStatistics.length - 1) {
      return this.setSliderValue(gasPriceStatistics.length - 1, bnGasPrice);
    }

    const priceA = gasPriceStatistics[minIndex];
    const priceB = gasPriceStatistics[minIndex + 1];

    const sliderValueDec = bnGasPrice
      .minus(priceA)
      .dividedBy(priceB.minus(priceA))
      .toNumber();

    const sliderValue = minIndex + sliderValueDec;
    this.setSliderValue(sliderValue, bnGasPrice);
  }

  setSliderValue (value, gasPrice = this.state.gasPrice) {
    const { gasPriceStatistics } = this.props;

    const sliderValue = Math.min(value, gasPriceStatistics.length - 1);

    this.setState(
      { sliderValue, gasPrice },
      () => this.updateGasPointChart(this.state, gasPrice)
    );
  }

  updateGasPointChart (state = this.state, gasPriceIn = this.state.gasPrice) {
    const bubbleChartData = { ...BUBBLE_CHART_DATA };
    const { sliderValue } = state;

    const gasPrice = gasPriceIn || this.props.gasPrice;

    if (gasPrice) {
      bubbleChartData.datasets[0].data[0] = {
        x: sliderValue,
        y: (new BigNumber(gasPrice)).toNumber(),
        r: 5
      };
    }

    this.setState({ bubbleChartData }, () => {
      if (this.refs.bubbleChart) {
        this.refs.bubbleChart.chart_instance.update();
      }
    });
  }

  onClickGasPrice = (items) => {
    const index = items.shift()._index;
    this.onEditGasPriceSlider(null, index + 0.5);
  }

  onEditGasPriceSlider = (event, sliderValue) => {
    const { gasPriceStatistics } = this.props;

    const gasPriceAIdx = Math.floor(sliderValue);
    const gasPriceBIdx = gasPriceAIdx + 1;

    if (gasPriceBIdx === gasPriceStatistics.length) {
      const gasPrice = gasPriceStatistics[gasPriceAIdx];
      this.props.onChange(event, gasPrice);
      return;
    }

    const gasPriceA = gasPriceStatistics[gasPriceAIdx];
    const gasPriceB = gasPriceStatistics[gasPriceBIdx];

    const mult = Math.round((sliderValue % 1) * 100) / 100;
    const gasPrice = gasPriceA.plus(gasPriceB.minus(gasPriceA).times(mult));

    this.setSliderValue(sliderValue, gasPrice);
    this.props.onChange(event, gasPrice);
  }
}
