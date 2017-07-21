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

import { observer } from 'mobx-react';
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import styles from '../shapeshift.css';

@observer
export default class ErrorStep extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { error } = this.props.store;

    return (
      <div className={ styles.body }>
        <div className={ styles.info }>
          <FormattedMessage
            id='shapeshift.errorStep.info'
            defaultMessage='The funds shifting via {shapeshiftLink} failed with a fatal error on the exchange. The error message received from the exchange is as follow:'
            values={ {
              shapeshiftLink: <a href='https://shapeshift.io' target='_blank'>ShapeShift.io</a>
            } }
          />
        </div>
        <div className={ styles.error }>
          { error.message }
        </div>
      </div>
    );
  }
}
