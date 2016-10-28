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

import {
  Area, AreaChart,
  Line,
  LineChart,
  Tooltip,
  XAxis, YAxis,
  ResponsiveContainer
} from 'recharts';

import Slider from 'material-ui/Slider';
import BigNumber from 'bignumber.js';

import componentStyles from './gasPriceSelector.css';
import mainStyles from '../transfer.css';

const styles = Object.assign({}, mainStyles, componentStyles);

class CustomTooltip extends Component {
  static propTypes = {
    type: PropTypes.string,
    payload: PropTypes.array,
    label: PropTypes.number,
    active: PropTypes.bool
  }

  render () {
    const { active } = this.props;

    if (!active) {
      return null;
    }

    const { payload } = this.props;

    const gasPrice = new BigNumber(payload[0].value || 0);

    return (
      <div>
        <p className='label'>{ gasPrice.toFormat(0) }</p>
      </div>
    );
  }
}

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
    selectedIndex: 0,

    chartData: []
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
    const { chartData, selectedIndex, sliderValue } = this.state;

    if (chartData.length === 0) {
      return null;
    }

    const height = 350;

    const axis = {
      xDomain: [0, chartData.length],
      yDomain: [0, this.props.gasPriceStatistics.slice(-1)[0].toNumber() * 1.1]
    };

    const N = chartData.length - 1;

    const offsets = {
      min: 100 * selectedIndex / N,
      max: 100 * (selectedIndex + 1) / N
    };

    const colors = {
      default: 'rgba(255, 99, 132, 0.2)',
      selected: 'rgba(255, 99, 132, 0.5)',
      hover: 'rgba(255, 99, 132, 0.35)',
      grid: 'rgba(255, 99, 132, 0.5)',
      line: 'rgb(255, 99, 132)',
      intersection: '#fff'
    };

    const gradientValues = chartData.map((d, index) => ({
      start: (100 * index / N) - 0.25,
      end: (100 * index / N) + 0.25,
      color: colors.grid
    }));

    gradientValues.push({
      start: offsets.min,
      end: offsets.max,
      color: colors.selected
    });

    const gradients = gradientValues
      .sort((a, b) => a.start - b.start)
      .reduce((current, datum) => {
        current.push({
          value: datum.start,
          color: colors.default
        });

        current.push({
          value: datum.start,
          color: datum.color
        });

        current.push({
          value: datum.end,
          color: datum.color
        });

        current.push({
          value: datum.end,
          color: colors.default
        });

        return current;
      }, [])
      .map((data, index) => (
        <stop key={ index } offset={ `${data.value}%` } stopColor={ data.color } />
      ));

    return (<div className={ styles.columns }>
      <div style={ { flex: 1, height } }>
        <div className={ styles.chart }>
          <ResponsiveContainer
            height={ height }
          >
            <LineChart
              data={ [
                { value: sliderValue },
                { value: sliderValue }
              ] }
              margin={ { top: 0, right: 0, left: 0, bottom: 0 } }
              layout='vertical'
            >

              <Line
                dataKey='value'
                stroke={ colors.intersection }
                isAnimationActive={ false }
                dot={ false }
              />

              <YAxis
                hide
                dataKey='value'
                type='category'
                domain={ axis.yDomain }
              />
              <XAxis
                hide
                type='number'
                domain={ [0, N] }
              />
            </LineChart>
          </ResponsiveContainer>
        </div>

        <div className={ styles.chart }>
          <ResponsiveContainer
            height={ height }
          >
            <AreaChart
              data={ chartData }
              margin={ { top: 0, right: 0, left: 0, bottom: 0 } }
            >
              <defs>
                <linearGradient
                  id='selectedColor'
                  x1='0' y1='0' x2='1' y2='0'
                >
                  { gradients }
                </linearGradient>
              </defs>

              <Area
                type='monotone'
                dataKey='value'
                stroke={ colors.line }
                fillOpacity={ 1 }
                fill='url(#selectedColor)'
                onClick={ this.onClickGasPrice }
              />

              <Tooltip
                wrapperStyle={ {
                  backgroundColor: 'rgba(0, 0, 0, 0.75)',
                  padding: '0 0.5em',
                  fontSize: '0.9em'
                } }
                content={ <CustomTooltip /> }
              />

              <XAxis
                hide
                dataKey='value'
                type='category'
                domain={ axis.xDomain }
              />
              <YAxis
                hide
                type='number'
                domain={ axis.yDomain }
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>);
  }

  computeChartsData () {
    const { gasPriceChartData } = this.state;
    const chartData = gasPriceChartData
      .map(value => ({ value }));

    this.setState({ chartData });
  }

  computeCharts (props = this.props) {
    const { gasPriceStatistics } = props;

    const gasPriceChartData = gasPriceStatistics
      .map(stat => stat.toNumber());

    this.setState(
      { gasPriceChartData },
      () => this.computeChartsData()
    );
  }

  updateSelectedBarChart (state = this.state) {
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
    const selectedIndex = Math.min(Math.floor(sliderValue), gasPriceStatistics.length - 2);

    this.setState({ sliderValue, gasPrice, selectedIndex });
  }

  onClickGasPrice = (event) => {
    const { left, right } = event.target.getBoundingClientRect();
    const { clientX } = event;

    const ratio = (clientX - left) / (right - left);
    const index = (this.props.gasPriceStatistics.length - 1) * ratio;

    this.onEditGasPriceSlider(null, index);
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
