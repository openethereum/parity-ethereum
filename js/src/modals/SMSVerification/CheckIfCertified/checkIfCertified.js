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
import SuccessIcon from 'material-ui/svg-icons/navigation/check';
import ErrorIcon from 'material-ui/svg-icons/alert/error-outline';

import styles from './checkIfCertified.css';

export default class CheckIfCertified extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.string.isRequired,
    contract: PropTypes.object.isRequired,
    onIsCertified: PropTypes.func.isRequired,
    onIsNotCertified: PropTypes.func.isRequired
  }

  state = {
    pending: false,
    isCertified: null
  };

  componentWillMount () {
    const { pending } = this.state;
    if (pending) {
      return;
    }
    this.setState({ pending: true });

    const { account, contract, onIsCertified, onIsNotCertified } = this.props;

    contract.instance.certified.call({}, [account])
      .then((isCertified) => {
        this.setState({ isCertified, pending: false });
        if (isCertified) {
          onIsCertified();
        } else {
          onIsNotCertified();
        }
      })
      .catch((err) => {
        console.error('error checking if certified', err);
      });
  }

  render () {
    const { pending, isCertified } = this.state;

    if (pending) {
      return (<p className={ styles.message }>Checking if your account is verified…</p>);
    }

    if (isCertified) {
      return (
        <div className={ styles.container }>
          <ErrorIcon />
          <p className={ styles.message }>Your account is already verified.</p>
        </div>
      );
    }
    return (
      <div className={ styles.container }>
        <SuccessIcon />
        <p className={ styles.message }>Your account is not verified yet.</p>
      </div>
    );
  }
}
