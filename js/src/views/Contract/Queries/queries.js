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
import RaisedButton from 'material-ui/RaisedButton';

import { Container, ContainerTitle } from '../../../ui';

import styles from '../contract.css';

export default class Queries extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    contract: PropTypes.object,
    values: PropTypes.object
  }

  state = {
    forms: {}
  }

  render () {
    const { contract } = this.props;

    if (!contract) {
      return null;
    }

    const queries = contract.functions
      .filter((fn) => fn.constant)
      .sort(this._sortEntries);

    const noInputQueries = queries
      .slice()
      .filter((fn) => fn.inputs.length === 0)
      .map((fn) => this.renderQuery(fn));

    const withInputQueries = queries
      .slice()
      .filter((fn) => fn.inputs.length > 0)
      .map((fn) => this.renderInputQuery(fn));

    return (
      <Container>
        <ContainerTitle title='queries' />
        <div className={ styles.methods }>
          { noInputQueries }
          { withInputQueries }
        </div>
      </Container>
    );
  }

  renderInputQuery (fn) {
    const { inputs } = fn;

    const inputsFields = inputs
      .map(input => this.renderInput(fn.name, input));

    const onClick = () => {
      const form = this.state.forms[fn.name];
      const inputsValue = inputs.map(input => {
        if (!form) return null;
        return form[input.name];
      });

      this.props
        .contract.instance[fn.name]
        .call({}, inputsValue)
        .then(results => {
          console.log(results);
        })
        .catch(e => {
          console.error(`sending ${fn.name} with params`, inputsValue, e);
        });
    };

    return (
      <div
        key={ fn.signature }
        >
        { fn.name }
        { inputsFields }
        <RaisedButton
          label='Execute'
          primary
          onClick={ onClick }
        />
      </div>
    );
  }

  renderInput (fnName, input) {
    const { name, kind } = input;
    const onChange = (event) => {
      const value = event.target.value;
      const { forms } = this.state;

      this.setState({
        forms: {
          ...forms,
          [fnName]: {
            ...forms[fnName],
            [ name ]: value
          }
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

  renderQuery (fn) {
    const { values } = this.props;

    return (
      <div
        key={ fn.signature }
        className={ styles.method }>
        <p>{ fn.name }</p>
        { this.renderValue(values[fn.name]) }
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

  _sortEntries (a, b) {
    return a.name.localeCompare(b.name);
  }
}
