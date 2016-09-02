import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { Actionbar, Container } from '../../../../ui';

import ToastrContainer from '../../components/ToastrContainer';
import { removeToast } from '../../actions/toastr';

import LoadingPage from '../LoadingPage';
import OfflinePage from '../OfflinePage';
import RequestsPage from '../RequestsPage';
import UnAuthorizedPage from '../UnAuthorizedPage';

import styles from './Root.css';

export class Root extends Component {
  static propTypes = {
    toastr: PropTypes.shape({
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
    const { toastr, actions } = this.props;

    return (
      <div>
        <Actionbar
          title='Parity Trusted Signer' />
        <Container>
          <div className={ styles.mainContainer }>
            { this.renderPage() }
          </div>
          <ToastrContainer
            toasts={ toastr.toasts }
            actions={ actions }
          />
        </Container>
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
)(Root);
