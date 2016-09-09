import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Badge, SignerIcon } from '../../ui';
import { Embedded as Signer } from '../Signer';

import imagesEthcoreBlock from '../../images/ethcore-block-blue.png';
import styles from './parityBar.css';

class ParityBar extends Component {
  static propTypes = {
    pending: PropTypes.array
  }

  state = {
    opened: false
  }

  render () {
    const { opened } = this.state;

    return opened
      ? this.renderExpanded()
      : this.renderBar();
  }

  renderBar () {
    const parityIcon = (
      <img
        src={ imagesEthcoreBlock }
        className={ styles.parityIcon } />
    );

    return (
      <div className={ styles.bar }>
        <div className={ styles.corner }>
          <Link to='/apps'>
            <FlatButton
              className={ styles.button }
              icon={ parityIcon }
              label={ this.renderLabel('Parity') }
              primary />
          </Link>
          <FlatButton
            className={ styles.button }
            icon={ <SignerIcon className={ styles.signerIcon } /> }
            label={ this.renderSignerLabel() }
            primary
            onTouchTap={ this.toggleDisplay } />
        </div>
      </div>
    );
  }

  renderExpanded () {
    return (
      <div className={ styles.expanded }>
        <div className={ styles.actions }>
          <FlatButton
            icon={ <ContentClear /> }
            label='Close'
            primary
            onTouchTap={ this.toggleDisplay } />
        </div>
        <Signer />
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
        <Badge
          color='red'
          className={ styles.labelBubble }
          value={ pending.length } />
      );
    }

    return this.renderLabel('Signer', bubble);
  }

  toggleDisplay = () => {
    const { opened } = this.state;

    this.setState({
      opened: !opened
    });
  }
}

function mapStateToProps (state) {
  const { pending } = state.signerRequests;

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
