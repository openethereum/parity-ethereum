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
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { Container, ContainerTitle } from '../../../../ui';

import { updateAppState } from '../../actions/signer';
import { isExtension } from '../../utils/extension';

class OfflinePage extends Component {
  static propTypes = {
    parityUrl: PropTypes.string.isRequired
  }

  render () {
    return (
      <Container>
        <ContainerTitle title='Offline' />
        <p>Could not connect to the node. Make sure Parity is running and Trusted Signer is enabled.</p>
        { this.renderInstallLink() }
      </Container>
    );
  }

  renderInstallLink () {
    if (!isExtension()) {
      return;
    }

    return (
      <p>
        If you do not have Parity installed yet, get it <a href='https://github.com/ethcore/parity/releases' target='_blank'>here</a>.
      </p>
    );
  }
}

function mapStateToProps (state) {
  return {
    parityUrl: state.signer.url
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ updateAppState }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(OfflinePage);
