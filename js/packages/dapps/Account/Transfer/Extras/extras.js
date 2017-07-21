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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { GasPriceEditor, Form, Input } from '@parity/ui';

import styles from '../transfer.css';

export default class Extras extends Component {
  static propTypes = {
    data: PropTypes.string,
    dataError: PropTypes.string,
    hideData: PropTypes.bool,
    gasStore: PropTypes.object.isRequired,
    isEth: PropTypes.bool,
    onChange: PropTypes.func,
    total: PropTypes.string,
    totalError: PropTypes.string
  };

  static defaultProps = {
    hideData: false
  };

  render () {
    const { gasStore, onChange } = this.props;

    return (
      <Form>
        { this.renderData() }
        <div className={ styles.gaseditor }>
          <GasPriceEditor
            store={ gasStore }
            onChange={ onChange }
          />
        </div>
      </Form>
    );
  }

  renderData () {
    const { isEth, data, dataError, hideData } = this.props;

    if (!isEth || hideData) {
      return null;
    }

    return (
      <Input
        error={ dataError }
        hint={
          <FormattedMessage
            id='transfer.advanced.data.hint'
            defaultMessage='the data to pass through with the transaction'
          />
        }
        label={
          <FormattedMessage
            id='transfer.advanced.data.label'
            defaultMessage='transaction data'
          />
        }
        onChange={ this.onEditData }
        value={ data }
      />
    );
  }

  onEditData = (event) => {
    this.props.onChange('data', event.target.value);
  }
}
