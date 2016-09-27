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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import Chip from 'material-ui/Chip';
import TextField from 'material-ui/TextField';
import LinearProgress from 'material-ui/LinearProgress';
import RaisedButton from 'material-ui/RaisedButton';

import styles from '../contract.css';

export default class InputQueries extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    contract: PropTypes.object.isRequired,
    inputs: PropTypes.array.isRequired,
    outputs: PropTypes.array.isRequired,
    name: PropTypes.string.isRequired
  }

  state = {
    isValid: true,
    results: [],
    values: {}
  }

  render () {
    const { inputs, name } = this.props;
    const { isValid } = this.state;

    const inputsFields = inputs
      .map(input => this.renderInput(input));

    return (<div>
      { name }
      { this.renderResults() }
      { inputsFields }
      <RaisedButton
        label='Execute'
        disabled={ !isValid }
        primary
        onClick={ this.onClick }
      />
    </div>);
  }

  renderResults () {
    const { results, isLoading } = this.state;
console.log(this.props.outputs);
    if (isLoading) {
      return (<LinearProgress mode='indeterminate' />);
    }

    if (!results || results.length < 1) return null;

    return results.map((result, index) => (
      <Chip
        key={ index }
      >
        { this.renderValue(result) }
      </Chip>
    ));
  }

  renderInput (input) {
    const { name, kind } = input;

    const onChange = (event) => {
      const value = event.target.value;
      const { values } = this.state;

      this.setState({
        values: {
          ...values,
          [ name ]: value
        }
      });
    };

    return (
      <div key={ name }>
        <TextField
          hintText={ kind.type }
          floatingLabelText={ name }
          floatingLabelFixed
          required
          onChange={ onChange }
        />
      </div>
    );
  }

  renderValue (value) {
    if (!value) return null;

    const { api } = this.context;
    let valueToDisplay = value.toString();

    if (api.util.isInstanceOf(value, BigNumber)) {
      valueToDisplay = value.toFormat(0);
    } else if (api.util.isArray(value)) {
      valueToDisplay = api.util.bytesToHex(value);
    }

    return (<Chip>{ valueToDisplay }</Chip>);
  }

  onClick = () => {
    const { values } = this.state;
    const { inputs, contract, name } = this.props;

    this.setState({
      isLoading: true,
      results: []
    });

    const inputValues = inputs.map(input => values[input.name]);

    contract
      .instance[name]
      .call({}, inputValues)
      .then(results => {
        this.setState({
          isLoading: false,
          results: [].concat(results)
        });
      })
      .catch(e => {
        console.error(`sending ${name} with params`, inputValues, e);
      });
  };
}
