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
import FlatButton from 'material-ui/FlatButton';
import { Card, CardActions, CardTitle, CardText } from 'material-ui/Card';

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

    return (
      <Card style={ {
        backgroundColor: 'rgba(48, 48, 48, 0.5)',
        margin: '1rem'
      } }>
        <CardTitle title={ name } />
        <CardText>
          { this.renderResults() }
          { inputsFields }
        </CardText>
        <CardActions>
          <FlatButton
            label='Execute'
            disabled={ !isValid }
            primary
            onTouchTap={ this.onClick } />
        </CardActions>
      </Card>
    );
  }

  renderResults () {
    const { results, isLoading } = this.state;
    const { outputs } = this.props;

    if (isLoading) {
      return (<LinearProgress mode='indeterminate' />);
    }

    if (!results || results.length < 1) return null;

    return outputs
      .map((out, index) => ({
        name: out.name,
        value: results[index]
      }))
      .map((out, index) => (<div key={ index }>
        <div>{ out.name }</div>
        <Chip labelStyle={ {
          userSelect: 'text'
        } }>
          { this.renderValue(out.value) }
        </Chip>
        <br />
      </div>));
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
    if (!value) return 'no data';

    const { api } = this.context;

    if (api.util.isInstanceOf(value, BigNumber)) {
      return value.toFormat(0);
    } else if (api.util.isArray(value)) {
      return api.util.bytesToHex(value);
    }

    return value.toString();
  }

  onClick = () => {
    const { values } = this.state;
    const { inputs, contract, name, outputs } = this.props;

    this.setState({
      isLoading: true,
      results: []
    });

    const inputValues = inputs.map(input => values[input.name]);

    contract
      .instance[name]
      .call({}, inputValues)
      .then(results => {
        if (outputs.length === 1) {
          results = [ results ];
        }

        this.setState({
          isLoading: false,
          results
        });
      })
      .catch(e => {
        console.error(`sending ${name} with params`, inputValues, e);
      });
  };
}
