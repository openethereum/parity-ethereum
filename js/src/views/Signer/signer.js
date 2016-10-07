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
import { connect } from 'react-redux';

import { Actionbar, Page } from '../../ui';

import LoadingPage from './containers/LoadingPage';
import RequestsPage from './containers/RequestsPage';

import styles from './signer.css';

export class Signer extends Component {
  static propTypes = {
    signer: PropTypes.shape({
      isLoading: PropTypes.bool.isRequired
    }).isRequired
  };

  render () {
    return (
      <div className={ styles.signer }>
        <Actionbar
          title='Trusted Signer' />
        <Page>
          { this.renderPage() }
        </Page>
      </div>
    );
  }

  renderPage () {
    const { isLoading } = this.props.signer;

    if (isLoading) {
      return (
        <LoadingPage />
      );
    }

    return (
      <RequestsPage />
    );
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {};
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Signer);
