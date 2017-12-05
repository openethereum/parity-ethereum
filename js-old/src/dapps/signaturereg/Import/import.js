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

import { api } from '../parity';
import { callRegister, postRegister } from '../services';
import Button from '../Button';

import styles from './import.css';

export default class Import extends Component {
  static propTypes = {
    instance: PropTypes.object.isRequired,
    visible: PropTypes.bool.isRequired,
    onClose: PropTypes.func.isRequired
  }

  state = {
    abi: null,
    abiParsed: null,
    abiError: 'Please add a valid ABI definition',
    functions: null,
    fnstate: {}
  }

  render () {
    const { visible, onClose } = this.props;
    const { abiError } = this.state;

    if (!visible) {
      return null;
    }

    return (
      <div className={ styles.modal }>
        <div className={ styles.overlay }>
          <div className={ styles.dialog }>
            <div className={ styles.header }>
              <div>abi import</div>
              <Button className={ styles.close } onClick={ onClose }>&times;</Button>
            </div>
            { abiError ? this.renderCapture() : this.renderRegister() }
          </div>
        </div>
      </div>
    );
  }

  renderCapture () {
    const { abiError } = this.state;

    return (
      <div className={ styles.body }>
        <div className={ styles.info }>
          Provide the ABI (Contract Interface) in the space provided below. Only non-constant functions (names &amp; types) will be imported, while constant functions and existing signatures will be ignored.
        </div>
        <div className={ styles.info }>
          <textarea rows='8' className={ styles.error } onChange={ this.onAbiEdit } />
          <div className={ styles.error }>
            { abiError }
          </div>
        </div>
      </div>
    );
  }

  renderRegister () {
    const count = this.countFunctions();
    let buttons = null;

    if (count) {
      buttons = (
        <div className={ styles.buttonrow }>
          <Button onClick={ this.onRegister }>
            register functions
          </Button>
        </div>
      );
    }

    return (
      <div className={ styles.body }>
        <div className={ styles.info }>
          The following functions have been extracted from the ABI provided and the state has been determined from interacting with the signature contract.
        </div>
        <div className={ styles.info }>
          <div className={ styles.fnkeys }>
            { this.renderFunctions() }
          </div>
        </div>
        <div className={ styles.info }>
          { count || 'no' } functions available for registration
        </div>
        { buttons }
      </div>
    );
  }

  renderFunctions () {
    const { functions, fnstate } = this.state;

    if (!functions) {
      return null;
    }

    return functions.map((fn) => {
      if (fn.constant) {
        fnstate[fn.signature] = 'fnconstant';
      } else if (!fnstate[fn.signature]) {
        this.testFunction(fn);
      }

      return (
        <div key={ fn.signature } className={ styles[fnstate[fn.signature] || 'fnunknown'] }>
          { fn.id }
        </div>
      );
    });
  }

  sortFunctions = (a, b) => {
    return (a.name || '').localeCompare(b.name || '');
  }

  countFunctions () {
    const { functions, fnstate } = this.state;

    if (!functions) {
      return 0;
    }

    return functions.filter((fn) => fnstate[fn.signature] === 'fntodo').length;
  }

  testFunction (fn) {
    const { instance } = this.props;
    const { fnstate } = this.state;

    callRegister(instance, fn.id)
      .then((result) => {
        fnstate[fn.signature] = result ? 'fntodo' : 'fnexists';
        this.setState(fnstate);
      })
      .catch((error) => {
        console.error(error);
      });
  }

  onAbiEdit = (event) => {
    let functions = null;
    let abiError = null;
    let abiParsed = null;
    let abi = null;

    try {
      abiParsed = JSON.parse(event.target.value);
      functions = api.newContract(abiParsed).functions.sort(this.sortFunctions);
      abi = JSON.stringify(abiParsed);
    } catch (error) {
      console.error('onAbiEdit', error);
      abiError = error.message;
    }

    this.setState({
      functions,
      abiError,
      abiParsed,
      abi
    });
  }

  onRegister = () => {
    const { instance, onClose } = this.props;
    const { functions, fnstate } = this.state;

    return Promise
      .all(
        functions
          .filter((fn) => !fn.constant)
          .filter((fn) => fnstate[fn.signature] === 'fntodo')
          .map((fn) => postRegister(instance, fn.id, {}))
      )
      .then(() => {
        onClose();
      })
      .catch((error) => {
        console.error('onRegister', error);
      });
  }
}
