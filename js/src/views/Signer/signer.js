import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import { Actionbar, Page } from '../../ui';

import LoadingPage from './containers/LoadingPage';
import OfflinePage from './containers/OfflinePage';
import RequestsPage from './containers/RequestsPage';
import UnAuthorizedPage from './containers/UnAuthorizedPage';

import styles from './signer.css';

export class Signer extends Component {
  static propTypes = {
    signer: PropTypes.shape({
      isLoading: PropTypes.bool.isRequired,
      isConnected: PropTypes.bool.isRequired,
      isNodeRunning: PropTypes.bool.isRequired
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
    const { isLoading, isConnected, isNodeRunning } = this.props.signer;

    if (isLoading) {
      return (
        <LoadingPage />
      );
    } else if (!isNodeRunning) {
      return (
        <OfflinePage />
      );
    } else if (!isConnected) {
      return (
        <UnAuthorizedPage />
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
