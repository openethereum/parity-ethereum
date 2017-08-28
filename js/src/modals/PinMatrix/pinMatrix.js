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

  constructor () {
    super();

    this.state = {
      passcode: '',
      failureMessage: ''
    };

    this.pinMatrix = [7, 8, 9, 4, 5, 6, 1, 2, 3];
  }

  render () {
    const { failureMessage, passcode } = this.state;
    const { device } = this.props;

    return (
      <div className={ styles.pinMatrix }>

        <div className={ styles.overlay } />

        <div className={ styles.modal }>
          <div className={ styles.body }>
            <div id={ styles.title }>
              <span>
                <FormattedMessage
                  id='pinMatrix.enterPin'
                  defaultMessage='Please enter the pin for your {manufacturer} hardware wallet'
                  values={ {
                    manufacturer: device.manufacturer
                  } }
                />
              </span>
            </div>
            <div id={ styles.passcodeBoxes }>
              {this.renderPasscodeBox()}
            </div>

            <div id={ styles.pin }>
              {passcode.replace(/./g, '*')}
              {
                (passcode.length)
                  ? <div id={ styles.clearThik } onClick={ this.removeDigit } />
                  : null
              }
            </div>
            <div>
              <span className={ styles.button } id={ styles.submit } onClick={ this.submit }>Submit</span>
            </div>
            <div id={ styles.error }>
              { failureMessage }
            </div>
          </div>
        </div>

      </div>
    );
  }

  renderPasscodeBox () {
    return Array.apply(null, Array(9)).map((box, index) => {
      let addDigit = () => this.addDigit(this.pinMatrix[index]);

      return (
        <div
          className={ styles.passcodeBox }
          onClick={ addDigit }
          key={ index }
        >
          <div className={ styles.passcodeBall } />
        </div>
      );
    });
  }

  addDigit = (digit) => {
    if (this.state.passcode.length > 8) {
      return;
    }
    this.setState({
      passcode: this.state.passcode + digit
    });
  }

  removeDigit = () => {
    this.setState({
      passcode: this.state.passcode.slice(0, -1)
    });
  }

  clearPasscode = () => {
    this.setState({
      passcode: ''
    });
  }

  submit = () => {
    const { device, store } = this.props;
    const { passcode } = this.state;

    store.pinMatrixAck(device, passcode)
      .then((status) => {
        if (!status) {
          this.setState({
            passcode: '',
            failureMessage: (
              <FormattedMessage
                id='pinMatrix.label.failureMessage'
                defaultMessage='Wrong pin, try again.'
              />
            )
          });
        } else {
          this.setState({
            passcode: '',
            failureMessage: ''
          });
        }
      });
  }
}
