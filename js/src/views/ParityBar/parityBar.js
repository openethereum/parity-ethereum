import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';

import imagesEthcoreBlock from '../../images/ethcore-block.png';
import styles from './parityBar.css';

class ParityBar extends Component {
  static propTypes = {
    pending: PropTypes.array
  }

  render () {
    return (
      <div className={ styles.bar }>
        <div className={ styles.corner }>
          <a
            className={ styles.link }
            href='/#/apps'>
            <img src={ imagesEthcoreBlock } />
            { this.renderLabel('Parity') }
          </a>
          <a
            className={ styles.link }
            href='/#/signer'>
            <ActionFingerprint />
            { this.renderSignerLabel() }
          </a>
        </div>
      </div>
    );
  }

  renderLabel (name, bubble) {
    return (
      <div className={ styles.label }>
        <div className={ styles.labelText }>
          { name }
        </div>
        { bubble }
      </div>
    );
  }

  renderSignerLabel () {
    const { pending } = this.props;
    let bubble = null;

    if (pending && pending.length) {
      bubble = (
        <div className={ styles.labelBubble }>
          { pending.length }
        </div>
      );
    }

    return this.renderLabel('Signer', bubble);
  }
}

function mapStateToProps (state) {
  const { pending } = state.requests;

  return {
    pending
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(ParityBar);
