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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import styles from './pinMatrix.css';

export default class PinMatrix extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired,
    device: PropTypes.object.isRequired
  }

  state = {
    passcode: '',
    failureMessage: ''
  }

  pinMatrix = [7, 8, 9, 4, 5, 6, 1, 2, 3]

  render () {
    const { passcode, failureMessage } = this.state;
    const { device } = this.props;

    return (
      <div className={ styles.overlay }>
        <div className={ styles.body }>
          <FormattedMessage
            id='pinMatrix.enterPin'
            defaultMessage='Please enter the pin for your {manufacturer} hardware wallet'
            values={ {
              manufacturer: device.manufacturer
            } }
          />
          <div className={ styles.passcodeBoxes }>
            {this.renderPasscodeBox()}
          </div>

          <div className={ styles.pin }>
            {passcode.replace(/./g, '*')}
            {
              passcode.length
                ? <div className={ styles.clearThik } onClick={ this.handleRemoveDigit } />
                : null
            }
          </div>
          <span
            className={ `${styles.button} ${styles.submit}` }
            onClick={ this.handleSubmit }
          >
            Submit
          </span>
          <div className={ styles.error }>
            { failureMessage }
          </div>
        </div>
      </div>
    );
  }

  handleAddDigit = (ev) => {
    const index = ev.currentTarget.getAttribute('data-index');
    const digit = this.pinMatrix[index];
    const { passcode } = this.state;

    if (passcode.length > 8) {
      return;
    }

    this.setState({
      passcode: passcode + digit
    });
  }

  renderPasscodeBox () {
    return Array.apply(null, Array(9)).map((box, index) => {
      return (
        <button
          className={ styles.passcodeBox }
          onClick={ this.handleAddDigit }
          data-index={ index }
          key={ index }
        >
          <div className={ styles.passcodeBall } />
        </button>
      );
    });
  }

  handleRemoveDigit = () => {
    this.setState({
      passcode: this.state.passcode.slice(0, -1)
    });
  }

  handleSubmit = () => {
    const { device, store } = this.props;
    const { passcode } = this.state;

    store.pinMatrixAck(device, passcode)
      .then((status) => {
        const passcode = '';
        const failureMessage = status ? '' : (
          <FormattedMessage
            id='pinMatrix.label.failureMessage'
            defaultMessage='Wrong pin, try again.'
          />
        );

        this.setState({ passcode, failureMessage });
      })
      .catch(err => {
        this.setState({
          failureMessage: err.toString()
        });
      });
  }
}
