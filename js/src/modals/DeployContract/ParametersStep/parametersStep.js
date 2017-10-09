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

import { Form, TypedInput } from '~/ui';
import { parseAbiType } from '~/util/abi';

import styles from '../deployContract.css';

export default class ParametersStep extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    onParamsChange: PropTypes.func.isRequired,

    inputs: PropTypes.array,
    params: PropTypes.array,
    paramsError: PropTypes.array
  };

  render () {
    return (
      <Form>
        { this.renderConstructorInputs() }
      </Form>
    );
  }

  renderConstructorInputs () {
    const { params, paramsError } = this.props;
    const { inputs } = this.props;

    if (!inputs || !inputs.length) {
      return null;
    }

    const inputsComponents = inputs.map((input, index) => {
      const onChange = (value) => this.onParamChange(index, value);

      const label = `${input.name ? `${input.name}: ` : ''}${input.type}`;
      const value = params[index];
      const error = paramsError[index];
      const param = parseAbiType(input.type);

      return (
        <div key={ index } className={ styles.funcparams }>
          <TypedInput
            error={ error }
            isEth={ false }
            label={ label }
            onChange={ onChange }
            param={ param }
            value={ value }
          />
        </div>
      );
    });

    return (
      <div className={ styles.parameters }>
        <p>
          <FormattedMessage
            id='deployContract.parameters.choose'
            defaultMessage='Choose the contract parameters'
          />
        </p>
        { inputsComponents }
      </div>
    );
  }

  onParamChange = (index, value) => {
    const { params, onParamsChange } = this.props;

    params[index] = value;
    onParamsChange(params);
  }
}
