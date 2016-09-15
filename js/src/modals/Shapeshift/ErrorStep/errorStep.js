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

import styles from '../shapeshift.css';

export default class ErrorStep extends Component {
  static propTypes = {
    error: PropTypes.shape({
      fatal: PropTypes.bool,
      message: PropTypes.string.isRequired
    }).isRequired
  }

  render () {
    const { error } = this.props;

    return (
      <div className={ styles.body }>
        <div className={ styles.info }>
          The funds shifting via <a href='https://shapeshift.io' target='_blank'>ShapeShift.io</a> failed with a fatal error on the exchange. The error message received from the exchange is as follow:
        </div>
        <div className={ styles.error }>
          { error.message }
        </div>
      </div>
    );
  }
}
