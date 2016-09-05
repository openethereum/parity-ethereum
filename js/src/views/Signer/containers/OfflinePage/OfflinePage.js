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
