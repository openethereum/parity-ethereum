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
import { Link } from 'react-router';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Badge, ContainerTitle, SignerIcon } from '../../ui';
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
        <div className={ styles.header }>
          <div className={ styles.title }>
            <ContainerTitle title='Parity Signer: Pending' />
          </div>
          <div className={ styles.actions }>
            <FlatButton
              icon={ <ContentClear /> }
              label='Close'
              primary
              onTouchTap={ this.toggleDisplay } />
          </div>
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
