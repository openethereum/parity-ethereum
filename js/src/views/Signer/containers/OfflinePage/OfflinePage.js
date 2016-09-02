import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { updateAppState } from '../../actions/signer';
import { isExtension } from '../../utils/extension';

class OfflinePage extends Component {
  static propTypes = {
    parityUrl: PropTypes.string.isRequired
  }

  render () {
    return (
      <div>
        <h2>Offline</h2>
        <p>Couldn't connect to the node. Make sure Parity is running and Trusted Signer is enabled.</p>
        { this.renderInstallLink() }
      </div>
    );
  }

  renderInstallLink () {
    if (!isExtension()) {
      return;
    }

    return (
      <p>
        If you don't have Parity installed yet, get it <a href='https://github.com/ethcore/parity/releases' target='_blank'>here</a>.
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
