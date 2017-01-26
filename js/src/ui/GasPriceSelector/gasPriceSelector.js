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
import { Slider } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import { Bar, BarChart, ResponsiveContainer, Scatter, ScatterChart, Tooltip, XAxis, YAxis } from 'recharts';

import CustomBar from './CustomBar';
import CustomCursor from './CustomCursor';
import CustomShape from './CustomShape';
import CustomTooltip from './CustomTooltip';

import { COLORS, countModifier } from './util';

import styles from './gasPriceSelector.css';

const TOOL_STYLE = {
  color: 'rgba(255,255,255,0.5)',
  backgroundColor: 'rgba(0, 0, 0, 0.75)',
  padding: '0 0.5em',
  fontSize: '0.75em'
};

export default class GasPriceSelector extends Component {
  static propTypes = {
    histogram: PropTypes.object.isRequired,
    onChange: PropTypes.func.isRequired,
    price: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.object
    ])
  }

  state = {
    price: null,
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
    this.setprice();
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.price !== this.props.price) {
      this.setprice(nextProps);
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

  renderChart () {
    const { histogram } = this.props;
    const { chartData, sliderValue, selectedIndex } = this.state;

    if (chartData.values.length === 0) {
      return null;
    }

    const height = 196;
    const countIndex = Math.max(0, Math.min(selectedIndex, histogram.counts.length - 1));
    const selectedCount = countModifier(histogram.counts[countIndex]);

    return (
      <div className={ styles.chartRow }>
        <div style={ { flex: 1, height } }>

          <div className={ styles.chart }>
            <ResponsiveContainer height={ height }>
              <ScatterChart margin={ { top: 0, right: 0, left: 0, bottom: 0 } }>
                <Scatter
                  data={ [
                    { x: sliderValue, y: 0 },
                    { x: sliderValue, y: selectedCount },
                    { x: sliderValue, y: chartData.yDomain[1] }
                  ] }
                  isAnimationActive={ false }
                  line
                  shape={
                    <CustomShape showValue={ selectedCount } />
                  }
                />
                <XAxis
                  dataKey='x'
                  domain={ [0, 1] }
                  hide
                  height={ 0 }
                />
                <YAxis
                  dataKey='y'
                  domain={ chartData.yDomain }
                  hide
                  width={ 0 }
                />
              </ScatterChart>
            </ResponsiveContainer>
          </div>

          <div className={ styles.chart }>
            <ResponsiveContainer height={ height }>
              <BarChart
                barCategoryGap={ 1 }
                data={ chartData.values }
                margin={ { top: 0, right: 0, left: 0, bottom: 0 } }
                ref='barChart'
              >
                <Bar
                  dataKey='value'
                  onClick={ this.onClickprice }
                  shape={ <CustomBar selected={ selectedIndex } onClick={ this.onClickprice } /> }stroke={ COLORS.line }
                />
                <Tooltip
                  content={ <CustomTooltip histogram={ histogram } /> }
                  cursor={ this.renderCustomCursor() }
                  wrapperStyle={ TOOL_STYLE }
                />
                <XAxis
                  dataKey='index'
                  domain={ chartData.xDomain }
                  hide
                  type='category'
                />
                <YAxis
                  domain={ chartData.yDomain }
                  hide
                  type='number'
                />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    );
  }

  renderSlider () {
    const { sliderValue } = this.state;

    return (
      <div className={ styles.sliderRow }>
        <Slider
          min={ 0 }
          max={ 1 }
          value={ sliderValue }
          onChange={ this.onEditpriceSlider }
          style={ {
            flex: 1,
            padding: '0 0.3em'
          } }
          sliderStyle={ {
            marginBottom: 12
          } }
        />
      </div>
    );
  }

  renderCustomCursor = () => {
    const { histogram } = this.props;
    const { chartData } = this.state;

    return (
      <CustomCursor
        counts={ histogram.counts }
        getIndex={ this.getBarHoverIndex }
        onClick={ this.onClickprice }
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
    const { priceChartData } = this.state;
    const { histogram } = this.props;

    const values = priceChartData
      .map((value, index) => ({ value, index }));

    const N = values.length - 1;
    const maxGasCounts = countModifier(
      histogram
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
    const { histogram } = props;

    const priceChartData = histogram
      .counts
      .map(count => countModifier(count));

    this.setState(
      { priceChartData },
      () => this.computeChartsData()
    );
  }

  updateSelectedBarChart (state = this.state) {
  }

  setprice (props = this.props) {
    const { price, histogram } = props;

    // If no gas price yet...
    if (!price) {
      return this.setSliderValue(0.5);
    }

    const bnprice = (typeof price === 'string')
      ? new BigNumber(price)
      : price;

    // If gas price hasn't changed
    if (this.state.price && bnprice.equals(this.state.price)) {
      return;
    }

    const prices = histogram.bucketBounds;
    const startIndex = prices
      .findIndex(price => price.greaterThan(bnprice)) - 1;

    // price Lower than the max in histogram
    if (startIndex === -1) {
      return this.setSliderValue(0, bnprice);
    }

    // price Greater than the max in histogram
    if (startIndex === -2) {
      return this.setSliderValue(1, bnprice);
    }

    const priceA = prices[startIndex];
    const priceB = prices[startIndex + 1];

    const sliderValueDec = bnprice
      .minus(priceA)
      .dividedBy(priceB.minus(priceA))
      .toNumber();

    const sliderValue = (startIndex + sliderValueDec) / (prices.length - 1);

    this.setSliderValue(sliderValue, bnprice);
  }

  setSliderValue (value, price = this.state.price) {
    const { histogram } = this.props;

    const N = histogram.bucketBounds.length - 1;

    const sliderValue = Math.max(0, Math.min(value, 1));
    const selectedIndex = Math.floor(sliderValue * N);

    this.setState({ sliderValue, price, selectedIndex });
  }

  onBarChartMouseUp = (event) => {
    console.log(event);
  }

  onClickprice = (bar) => {
    const { index } = bar;

    const ratio = (index + 0.5) / (this.state.chartData.N + 1);

    this.onEditpriceSlider(null, ratio);
  }

  onEditpriceSlider = (event, sliderValue) => {
    const { histogram } = this.props;

    const prices = histogram.bucketBounds;
    const N = prices.length - 1;
    const priceAIdx = Math.floor(sliderValue * N);
    const priceBIdx = priceAIdx + 1;

    if (priceBIdx === N + 1) {
      const price = prices[priceAIdx].round();

      this.props.onChange(event, price);
      return;
    }

    const priceA = prices[priceAIdx];
    const priceB = prices[priceBIdx];

    const mult = Math.round((sliderValue % 1) * 100) / 100;
    const price = priceA
      .plus(priceB.minus(priceA).times(mult))
      .round();

    this.setSliderValue(sliderValue, price);
    this.props.onChange(event, price.toFixed());
  }
}
