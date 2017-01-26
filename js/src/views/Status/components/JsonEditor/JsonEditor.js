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
import { isEqual } from 'lodash';
import formatJson from 'format-json';

import styles from './JsonEditor.css';

export default class JsonEditor extends Component {
  constructor (...args) {
    super(...args);
    let { value } = this.props;

    value = formatJson.plain(value);
    this.state = { value };
  }

  componentDidMount () {
    const mockedEvt = { target: { value: this.state.value } };

    this.onChange(mockedEvt);
  }

  componentWillReceiveProps (nextProps) {
    let { value } = nextProps;

    if (!isEqual(value, this.props.value)) {
      value = formatJson.plain(value);
      this.setState({ value });
    }
  }

  render () {
    let errorClass = this.state.error ? styles.error : '';

    return (
      <div className='row'>
        <textarea
          onChange={ this.onChange }
          className={ `${styles.editor} ${errorClass}` }
          value={ this.state.value }
        />
        { this.renderError() }
      </div>
    );
  }

  renderError () {
    const { error } = this.state;

    if (!error) {
      return;
    }

    return (
      <div className={ styles.errorMsg }>{ error }</div>
    );
  }

  onChange = evt => {
    const { value } = evt.target;
    let parsed;
    let error;

    try {
      parsed = JSON.parse(value);
      error = null;
    } catch (err) {
      parsed = null;
      error = 'invalid json';
    }

    this.setState({
      value,
      error
    });

    this.props.onChange(parsed, error);
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired,
    value: PropTypes.object
  }
}
