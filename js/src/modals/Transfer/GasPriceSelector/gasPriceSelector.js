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
  Bar, BarChart,
  Rectangle,
  Scatter, ScatterChart,
  Tooltip,
  XAxis, YAxis,
  Dot,
  ResponsiveContainer
} from 'recharts';

import Slider from 'material-ui/Slider';
import BigNumber from 'bignumber.js';

import componentStyles from './gasPriceSelector.css';
import mainStyles from '../transfer.css';

const styles = Object.assign({}, mainStyles, componentStyles);

const COLORS = {
  default: 'rgba(255, 99, 132, 0.2)',
  selected: 'rgba(255, 99, 132, 0.5)',
  hover: 'rgba(255, 99, 132, 0.15)',
  grid: 'rgba(255, 99, 132, 0.5)',
  line: 'rgb(255, 99, 132)',
  intersection: '#fff'
};

const countModifier = (count) => {
  const val = count.toNumber ? count.toNumber() : count;
  return Math.log10(val + 1) + 0.1;
};

class CustomCursor extends Component {
  static propTypes = {
    x: PropTypes.number,
    y: PropTypes.number,
    width: PropTypes.number,
    height: PropTypes.number,
    onClick: PropTypes.func,
    getIndex: PropTypes.func,
    counts: PropTypes.array,
    yDomain: PropTypes.array
  }

  render () {
    const { x, y, width, height, getIndex, counts, yDomain } = this.props;

    const index = getIndex();

    if (index === -1) {
      return null;
    }

    const count = countModifier(counts[index]);
    const barHeight = (count / yDomain[1]) * (y + height);

    return (
      <g>
        <Rectangle
          x={ x }
          y={ 0 }
          width={ width }
          height={ height + y }
          fill='transparent'
          onClick={ this.onClick }
        />
        <Rectangle
          x={ x }
          y={ y + (height - barHeight) }
          width={ width }
          height={ barHeight }
          fill={ COLORS.hover }
          onClick={ this.onClick }
        />
      </g>
    );
  }

  onClick = () => {
    const { onClick, getIndex } = this.props;
    const index = getIndex();
    onClick({ index });
  }
}

class CustomBar extends Component {
  static propTypes = {
    selected: PropTypes.number,
    x: PropTypes.number,
    y: PropTypes.number,
    width: PropTypes.number,
    height: PropTypes.number,
    index: PropTypes.number,
    onClick: PropTypes.func
  }

  render () {
    const { x, y, selected, index, width, height, onClick } = this.props;

    const fill = selected === index
      ? COLORS.selected
      : COLORS.default;

    const borderWidth = 0.5;
    const borderColor = 'rgba(255, 255, 255, 0.5)';

    return (
      <g>
        <Rectangle
          x={ x - borderWidth }
          y={ y }
          width={ borderWidth }
          height={ height }
          fill={ borderColor }
        />
        <Rectangle
          x={ x + width }
          y={ y }
          width={ borderWidth }
          height={ height }
          fill={ borderColor }
        />
        <Rectangle
          x={ x - borderWidth }
          y={ y - borderWidth }
          width={ width + borderWidth * 2 }
          height={ borderWidth }
          fill={ borderColor }
        />
        <Rectangle
          x={ x }
          y={ y }
          width={ width }
          height={ height }
          fill={ fill }
          onClick={ onClick }
        />
      </g>
    );
  }
}

class CustomizedShape extends Component {
  static propTypes = {
    showValue: PropTypes.number.isRequired,
    cx: PropTypes.number,
    cy: PropTypes.number,
    payload: PropTypes.object
  }

  render () {
    const { cx, cy, showValue, payload } = this.props;

    if (showValue !== payload.y) {
      return null;
    }

    return (
      <g>
        <Dot
          style={ { fill: 'white' } }
          cx={ cx }
          cy={ cy }
          r={ 5 }
        />
        <Dot
          style={ { fill: 'rgb(255, 99, 132)' } }
          cx={ cx }
          cy={ cy }
          r={ 3 }
        />
      </g>
    );
  }
}

class CustomTooltip extends Component {
  static propTypes = {
    gasPriceHistogram: PropTypes.shape({
      bucketBounds: PropTypes.array.isRequired,
      counts: PropTypes.array.isRequired
    }).isRequired,
    type: PropTypes.string,
    payload: PropTypes.array,
    label: PropTypes.number,
    active: PropTypes.bool
  }

  render () {
    const { active, label, gasPriceHistogram } = this.props;

    if (!active) {
      return null;
    }

    const index = label;

    const count = gasPriceHistogram.counts[index];
    const minGasPrice = gasPriceHistogram.bucketBounds[index];
    const maxGasPrice = gasPriceHistogram.bucketBounds[index + 1];

    return (
      <div>
        <p className='label'>
          { count.toNumber() } transactions
          with gas price set from
          <span> { minGasPrice.toFormat(0) } </span>
          to
          <span> { maxGasPrice.toFormat(0) } </span>
        </p>
      </div>
    );
  }
}

export default class GasPriceSelector extends Component {
  static propTypes = {
    gasPriceHistogram: PropTypes.shape({
      bucketBounds: PropTypes.array.isRequired,
      counts: PropTypes.array.isRequired
    }).isRequired,
    onChange: PropTypes.func.isRequired,

    gasPrice: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.object
    ])
  }

  state = {
    gasPrice: null,
    sliderValue: 0.5,
    selectedIndex: 0,

    chartData: {
      values: [],
      xDomain: [],
      yDomain: [],
      N: 0
    }
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

    return (<div className={ styles.columns }>
      <Slider
        min={ 0 }
        max={ 1 }
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
    const { gasPriceHistogram } = this.props;
    const { chartData, sliderValue, selectedIndex } = this.state;

    if (chartData.values.length === 0) {
      return null;
    }

    const height = 300;
    const countIndex = Math.max(0, Math.min(selectedIndex, gasPriceHistogram.counts.length - 1));
    const selectedCount = countModifier(gasPriceHistogram.counts[countIndex]);

    return (<div className={ styles.columns }>
      <div style={ { flex: 1, height } }>
        <div className={ styles.chart }>
          <ResponsiveContainer
            height={ height }
          >
            <ScatterChart
              margin={ { top: 0, right: 0, left: 0, bottom: 0 } }
            >
              <Scatter
                data={ [
                  { x: sliderValue, y: 0 },
                  { x: sliderValue, y: selectedCount },
                  { x: sliderValue, y: chartData.yDomain[1] }
                ] }
                shape={ <CustomizedShape showValue={ selectedCount } /> }
                line
                isAnimationActive={ false }
              />

              <XAxis
                hide
                height={ 0 }
                dataKey='x'
                domain={ [0, 1] }
              />
              <YAxis
                hide
                width={ 0 }
                dataKey='y'
                domain={ chartData.yDomain }
              />
            </ScatterChart>
          </ResponsiveContainer>
        </div>

        <div className={ styles.chart }>
          <ResponsiveContainer
            height={ height }
          >
            <BarChart
              data={ chartData.values }
              margin={ { top: 0, right: 0, left: 0, bottom: 0 } }
              barCategoryGap={ 1 }
              ref='barChart'
            >
              <Bar
                dataKey='value'
                stroke={ COLORS.line }
                onClick={ this.onClickGasPrice }
                shape={ <CustomBar selected={ selectedIndex } onClick={ this.onClickGasPrice } /> }
              />

              <Tooltip
                wrapperStyle={ {
                  backgroundColor: 'rgba(0, 0, 0, 0.75)',
                  padding: '0 0.5em',
                  fontSize: '0.9em'
                } }
                cursor={ this.renderCustomCursor() }
                content={ <CustomTooltip gasPriceHistogram={ gasPriceHistogram } /> }
              />

              <XAxis
                hide
                dataKey='index'
                type='category'
                domain={ chartData.xDomain }
              />
              <YAxis
                hide
                type='number'
                domain={ chartData.yDomain }
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>);
  }

  renderCustomCursor = () => {
    const { gasPriceHistogram } = this.props;
    const { chartData } = this.state;

    return (
      <CustomCursor
        getIndex={ this.getBarHoverIndex }
        onClick={ this.onClickGasPrice }
        counts={ gasPriceHistogram.counts }
        yDomain={ chartData.yDomain }
      />
    );
  }

  getBarHoverIndex = () => {
    const { barChart } = this.refs;

    if (!barChart || !barChart.state) {
      return -1;
    }

    return barChart.state.activeTooltipIndex;
  }

  computeChartsData () {
    const { gasPriceChartData } = this.state;
    const { gasPriceHistogram } = this.props;

    const values = gasPriceChartData
      .map((value, index) => ({ value, index }));

    const N = values.length - 1;
    const maxGasCounts = countModifier(
      gasPriceHistogram
        .counts
        .reduce((max, count) => count.greaterThan(max) ? count : max, 0)
    );

    const xDomain = [0, N];
    const yDomain = [0, maxGasCounts * 1.1];

    const chartData = {
      values, N,
      xDomain, yDomain
    };

    this.setState({ chartData }, () => {
      this.updateSelectedBarChart();
    });
  }

  computeCharts (props = this.props) {
    const { gasPriceHistogram } = props;

    const gasPriceChartData = gasPriceHistogram
      .counts
      .map(count => countModifier(count));

    this.setState(
      { gasPriceChartData },
      () => this.computeChartsData()
    );
  }

  updateSelectedBarChart (state = this.state) {
  }

  setGasPrice (props = this.props) {
    const { gasPrice, gasPriceHistogram } = props;

    // If no gas price yet...
    if (!gasPrice) {
      return this.setSliderValue(0.5);
    }

    const bnGasPrice = (typeof gasPrice === 'string')
      ? new BigNumber(gasPrice)
      : gasPrice;

    // If gas price hasn't changed
    if (this.state.gasPrice && bnGasPrice.equals(this.state.gasPrice)) {
      return;
    }

    const gasPrices = gasPriceHistogram.bucketBounds;
    const startIndex = gasPrices
      .findIndex(price => price.greaterThan(bnGasPrice)) - 1;

    // gasPrice Lower than the max in histogram
    if (startIndex === -1) {
      return this.setSliderValue(0, bnGasPrice);
    }

    // gasPrice Greater than the max in histogram
    if (startIndex === -2) {
      return this.setSliderValue(1, bnGasPrice);
    }

    const priceA = gasPrices[startIndex];
    const priceB = gasPrices[startIndex + 1];

    const sliderValueDec = bnGasPrice
      .minus(priceA)
      .dividedBy(priceB.minus(priceA))
      .toNumber();

    const sliderValue = (startIndex + sliderValueDec) / (gasPrices.length - 1);
    this.setSliderValue(sliderValue, bnGasPrice);
  }

  setSliderValue (value, gasPrice = this.state.gasPrice) {
    const { gasPriceHistogram } = this.props;

    const N = gasPriceHistogram.bucketBounds.length - 1;

    const sliderValue = Math.max(0, Math.min(value, 1));
    const selectedIndex = Math.floor(sliderValue * N);

    this.setState({ sliderValue, gasPrice, selectedIndex });
  }

  onBarChartMouseUp = (event) => {
    console.log(event);
  }

  onClickGasPrice = (bar) => {
    const { index } = bar;

    const ratio = (index + 0.5) / (this.state.chartData.N + 1);

    this.onEditGasPriceSlider(null, ratio);
  }

  onEditGasPriceSlider = (event, sliderValue) => {
    const { gasPriceHistogram } = this.props;

    const gasPrices = gasPriceHistogram.bucketBounds;
    const N = gasPrices.length - 1;
    const gasPriceAIdx = Math.floor(sliderValue * N);
    const gasPriceBIdx = gasPriceAIdx + 1;

    if (gasPriceBIdx === N + 1) {
      const gasPrice = gasPrices[gasPriceAIdx];
      this.props.onChange(event, gasPrice);
      return;
    }

    const gasPriceA = gasPrices[gasPriceAIdx];
    const gasPriceB = gasPrices[gasPriceBIdx];

    const mult = Math.round((sliderValue % 1) * 100) / 100;
    const gasPrice = gasPriceA.plus(gasPriceB.minus(gasPriceA).times(mult));

    this.setSliderValue(sliderValue, gasPrice);
    this.props.onChange(event, gasPrice);
  }
}
