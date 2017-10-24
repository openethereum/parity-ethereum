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

import { LinearProgress } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import ReactDOM from 'react-dom';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { hideRequest } from '~/redux/providers/requestsActions';
import { MethodDecoding, IdentityIcon, ScrollableText, ShortenedHash } from '~/ui';

import styles from './requests.css';

const ERROR_STATE = 'ERROR_STATE';
const DONE_STATE = 'DONE_STATE';
const WAITING_STATE = 'WAITING_STATE';

class Requests extends Component {
  static propTypes = {
    requests: PropTypes.object.isRequired,
    onHideRequest: PropTypes.func.isRequired
  };

  state = {
    extras: {}
  };

  render () {
    const { requests } = this.props;
    const { extras } = this.state;

    return (
      <div className={ styles.requests }>
        {
          Object
            .values(requests)
            .map((request) => this.renderRequest(request, extras[request.requestId]))
        }
      </div>
    );
  }

  renderRequest (request, extras = {}) {
    const { show, transaction } = request;

    if (!transaction) {
      return null;
    }

    const state = this.getTransactionState(request);
    const displayedTransaction = { ...transaction };

    // Don't show gas and gasPrice
    delete displayedTransaction.gas;
    delete displayedTransaction.gasPrice;

    const requestClasses = [ styles.request ];
    const statusClasses = [ styles.status ];
    const requestStyle = {};

    const handleHideRequest = () => {
      this.handleHideRequest(request.requestId);
    };

    if (state.type === ERROR_STATE) {
      statusClasses.push(styles.error);
    }

    if (!show) {
      requestClasses.push(styles.hide);
    }

    // Set the Request height (for animation) if found
    if (extras.height) {
      requestStyle.height = extras.height;
    }

    return (
      <div
        className={ requestClasses.join(' ') }
        key={ request.requestId }
        ref={ `request_${request.requestId}` }
        onClick={ handleHideRequest }
        style={ requestStyle }
      >
        <div className={ statusClasses.join(' ') }>
          { this.renderStatus(request) }
        </div>
        {
          state.type === ERROR_STATE
            ? null
            : (
              <LinearProgress
                max={ 6 }
                mode={ state.type === WAITING_STATE ? 'indeterminate' : 'determinate' }
                value={ state.type === DONE_STATE ? +request.blockHeight : 6 }
              />
            )
        }
        <div className={ styles.container }>
          <div
            className={ styles.identity }
            title={ transaction.from }
          >
            <IdentityIcon
              address={ transaction.from }
              inline
              center
              className={ styles.icon }
            />
          </div>
          <MethodDecoding
            address={ transaction.from }
            compact
            historic={ state.type === DONE_STATE }
            transaction={ displayedTransaction }
          />
        </div>
      </div>
    );
  }

  renderStatus (request) {
    const { error, transactionHash, transactionReceipt } = request;

    if (error) {
      return (
        <div
          className={ styles.inline }
          title={ error.message }
        >
          <FormattedMessage
            id='requests.status.error'
            defaultMessage='An error occured:'
          />
          <div className={ styles.fill }>
            <ScrollableText
              text={ error.text || error.message || error.toString() }
            />
          </div>
        </div>
      );
    }

    if (transactionReceipt) {
      return (
        <FormattedMessage
          id='requests.status.transactionMined'
          defaultMessage='Transaction mined at block #{blockNumber} ({blockHeight} confirmations)'
          values={ {
            blockHeight: (+request.blockHeight || 0).toString(),
            blockNumber: +transactionReceipt.blockNumber
          } }
        />
      );
    }

    if (transactionHash) {
      return (
        <div className={ styles.inline }>
          <FormattedMessage
            id='requests.status.transactionSent'
            defaultMessage='Transaction sent to network with hash'
          />
          <div className={ [ styles.fill, styles.hash ].join(' ') }>
            <ShortenedHash data={ transactionHash } />
          </div>
        </div>
      );
    }

    return (
      <FormattedMessage
        id='requests.status.waitingForSigner'
        defaultMessage='Waiting for authorization in the Parity Signer'
      />
    );
  }

  getTransactionState (request) {
    const { error, transactionReceipt } = request;

    if (error) {
      return { type: ERROR_STATE };
    }

    if (transactionReceipt) {
      return { type: DONE_STATE };
    }

    return { type: WAITING_STATE };
  }

  handleHideRequest = (requestId) => {
    const requestElement = ReactDOM.findDOMNode(this.refs[`request_${requestId}`]);

    // Try to get the request element height, to have a nice transition effect
    if (requestElement) {
      const { height } = requestElement.getBoundingClientRect();
      const prevExtras = this.state.extras;
      const nextExtras = {
        ...prevExtras,
        [ requestId ]: {
          ...prevExtras[requestId],
          height
        }
      };

      return this.setState({ extras: nextExtras }, () => {
        return this.props.onHideRequest(requestId);
      });
    }

    return this.props.onHideRequest(requestId);
  }
}

const mapStateToProps = (state) => {
  const { requests } = state;

  return { requests };
};

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onHideRequest: hideRequest
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Requests);
