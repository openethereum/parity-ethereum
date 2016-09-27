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

import RaisedButton from 'material-ui/RaisedButton';

import { REJECT_COUNTER_TIME } from '../../constants/constants';
import styles from './TransactionPendingFormReject.css';

export default class TransactionPendingFormReject extends Component {

  static propTypes = {
    onReject: PropTypes.func.isRequired,
    className: PropTypes.string,
    rejectCounterTime: PropTypes.number
  };

  static defaultProps = {
    rejectCounterTime: REJECT_COUNTER_TIME
  };

  state = {
    rejectCounter: this.props.rejectCounterTime
  }

  componentWillMount () {
    this.onInitCounter();
  }

  componentWillUnmount () {
    this.onResetCounter();
  }

  render () {
    const { rejectCounter } = this.state;
    const { onReject } = this.props;

    return (
      <div>
        <div className={ styles.rejectText }>
          Are you sure you want to reject transaction? <br />
          <strong>This cannot be undone</strong>
        </div>
        <RaisedButton
          onClick={ onReject }
          className={ styles.rejectButton }
          disabled={ rejectCounter > 0 }
          fullWidth
          label={ `Reject Transaction ${this.renderCounter()}` }
        />
      </div>
    );
  }

  renderCounter () {
    const { rejectCounter } = this.state;
    if (!rejectCounter) {
      return '';
    }
    return `(${rejectCounter})`;
  }

  onInitCounter () {
    this.rejectInterval = setInterval(() => {
      let { rejectCounter } = this.state;
      if (rejectCounter === 0) {
        return clearInterval(this.rejectInterval);
      }
      this.setState({ rejectCounter: rejectCounter - 1 });
    }, 1000);
  }

  onResetCounter () {
    clearInterval(this.rejectInterval);
    this.setState({
      rejectCounter: this.props.rejectCounterTime
    });
  }
}
