import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { Actionbar, Page } from '../../ui';

import { removeToast } from './actions/toastr';
import { ToastrContainer } from './components';
import LoadingPage from './containers/LoadingPage';
import OfflinePage from './containers/OfflinePage';
import RequestsPage from './containers/RequestsPage';
import UnAuthorizedPage from './containers/UnAuthorizedPage';

import styles from './signer.css';

export class Signer extends Component {
  static propTypes = {
    signerToastr: PropTypes.shape({
      toasts: PropTypes.array.isRequired
    }).isRequired,
    actions: PropTypes.shape({
      removeToast: PropTypes.func.isRequired
    }).isRequired,
    signer: PropTypes.shape({
      isLoading: PropTypes.bool.isRequired,
      isConnected: PropTypes.bool.isRequired,
      isNodeRunning: PropTypes.bool.isRequired
    }).isRequired
  };

  render () {
    const { signerToastr, actions } = this.props;

    return (
      <div className={ styles.signer }>
        <Actionbar
          title='Trusted Signer' />
        <Page>
          { this.renderPage() }
        </Page>
        <ToastrContainer
          toasts={ signerToastr.toasts }
          actions={ actions }
        />
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
  return {
    actions: bindActionCreators({ removeToast }, dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Signer);
