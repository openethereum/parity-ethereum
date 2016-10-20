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

import ReactTooltip from 'react-tooltip';
import DescriptionIcon from 'material-ui/svg-icons/action/description';
import GasIcon from 'material-ui/svg-icons/maps/local-gas-station';
import TimeIcon from 'material-ui/svg-icons/device/access-time';
import moment from 'moment';

import styles from './TransactionSecondaryDetails.css';

import * as tUtil from '../util/transaction';

export default class TransactionSecondaryDetails extends Component {

  static propTypes = {
    id: PropTypes.object.isRequired,
    date: PropTypes.instanceOf(Date),
    data: PropTypes.string, // hex
    gasPriceEthmDisplay: PropTypes.string,
    gasToDisplay: PropTypes.string,
    className: PropTypes.string
  };

  state = {
    isDataExpanded: false
  };

  render () {
    const className = this.props.className || '';

    return (
      <div className={ className }>
        <div className={ styles.iconsContainer }>
          { this.renderGasPrice() }
          { this.renderData() }
          { this.renderDate() }
        </div>
        <div className={ styles.expandedContainer }>
          { this.renderDataExpanded() }
        </div>
      </div>
    );
  }

  renderGasPrice () {
    if (!this.props.gasPriceEthmDisplay && !this.props.gasToDisplay) return null;

    const { id } = this.props;
    const { gasPriceEthmDisplay, gasToDisplay } = this.props;
    return (
      <div
        data-tip
        data-place='right'
        data-for={ 'gasPrice' + id }
        data-effect='solid'
      >
        <span className={ styles.gasPrice }>
          <GasIcon />
          { gasPriceEthmDisplay } <small>ETH/MGAS</small>
        </span>
        { /* dynamic id required in case there are multple transactions in page */ }
        <ReactTooltip id={ 'gasPrice' + id }>
          Cost of 1,000,000 units of gas. This transaction will use up to <strong>{ gasToDisplay }</strong> <small>MGAS</small>.
        </ReactTooltip>
      </div>
    );
  }

  renderData () {
    if (!this.props.data) return null;

    const { data, id } = this.props;
    let dataToDisplay = this.noData() ? 'no data' : tUtil.getShortData(data);
    const noDataClass = this.noData() ? styles.noData : '';
    return (
      <div
        className={ `${styles.data} ${noDataClass}` }
        onClick={ this.toggleDataExpanded }
        data-tip
        data-place='right'
        data-for={ 'data' + id }
        data-class={ styles.dataTooltip }
        data-effect='solid'
      >
        <DescriptionIcon />
        { dataToDisplay }
        { /* dynamic id required in case there are multple transactions in page */ }
        <ReactTooltip id={ 'data' + id }>
          <strong>Extra data for the transaction: </strong>
          <br />
          { dataToDisplay }.
          <br />
          { this.noData() ? '' : <strong>Click to expand.</strong> }
        </ReactTooltip>
      </div>
    );
  }

  renderDate () {
    const { date, id } = this.props;

    const dateToDisplay = moment(date).fromNow();
    const fullDate = moment(date).format('LL LTS');

    return (
      <div
        className={ styles.date }
        data-tip
        data-place='right'
        data-for={ 'date' + id }
        data-class={ styles.dataTooltip }
        data-effect='solid'
      >
        <TimeIcon />
        { dateToDisplay }
        { /* dynamic id required in case there are multple transactions in page */ }
        <ReactTooltip id={ 'date' + id }>
          <strong>Date of the request: </strong>
          <br />
          { fullDate }
        </ReactTooltip>
      </div>
    );
  }

  renderDataExpanded () {
    if (!this.props.data) return null;

    const { isDataExpanded } = this.state;
    const { data } = this.props;

    if (!isDataExpanded) {
      return;
    }

    return (
      <div className={ styles.expandedHelper }>
        <h3>Transaction's Data</h3>
        <code className={ styles.expandedData }>{ data }</code>
      </div>
    );
  }

  noData () {
    return this.props.data === '0x';
  }

  toggleDataExpanded = () => {
    if (this.noData()) {
      return;
    }
    this.setState({
      isDataExpanded: !this.state.isDataExpanded
    });
  }

}
