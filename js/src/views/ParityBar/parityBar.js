// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { Badge, Button, ContainerTitle, ParityBackground } from '~/ui';
import { Embedded as Signer } from '../Signer';

import imagesEthcoreBlock from '!url-loader!../../../assets/images/parity-logo-white-no-text.svg';
import styles from './parityBar.css';

class ParityBar extends Component {

  static propTypes = {
    dapp: PropTypes.bool,
    externalLink: PropTypes.string,
    pending: PropTypes.array
  };

  state = {
    opened: false
  }

  componentWillReceiveProps (nextProps) {
    const count = this.props.pending.length;
    const newCount = nextProps.pending.length;

    if (count === newCount) {
      return;
    }

    if (count < newCount) {
      this.setOpened(true);
    } else if (newCount === 0 && count === 1) {
      this.setOpened(false);
    }
  }

  setOpened (opened) {
    this.setState({ opened });

    if (!this.bar) {
      return;
    }

    // Fire up custom even to support having parity bar iframed.
    const event = new CustomEvent('parity.bar.visibility', {
      bubbles: true,
      detail: { opened }
    });

    this.bar.dispatchEvent(event);
  }

  onRef = (element) => {
    this.bar = element;
  }

  render () {
    const { opened } = this.state;

    return opened
      ? this.renderExpanded()
      : this.renderBar();
  }

  renderBar () {
    const { dapp } = this.props;

    if (!dapp) {
      return null;
    }

    const parityIcon = (
      <img
        src={ imagesEthcoreBlock }
        className={ styles.parityIcon } />
    );

    const parityButton = (
      <Button
        className={ styles.parityButton }
        icon={ parityIcon }
        label={ this.renderLabel('Parity') }
      />
    );

    return (
      <div
        className={ styles.bar }
        ref={ this.onRef }
      >
        <ParityBackground className={ styles.corner }>
          <div className={ styles.cornercolor }>
            { this.renderLink(parityButton) }
            <Button
              className={ styles.button }
              icon={ <ActionFingerprint /> }
              label={ this.renderSignerLabel() }
              onClick={ this.toggleDisplay }
            />
          </div>
        </ParityBackground>
      </div>
    );
  }

  renderLink (button) {
    const { externalLink } = this.props;

    if (!externalLink) {
      return (
        <Link to='/apps'>
          { button }
        </Link>
      );
    }

    return (
      <a
        href={ externalLink }
        target='_parent'
      >
        { button }
      </a>
    );
  }

  renderExpanded () {
    return (
      <div
        className={ styles.overlay }
        ref={ this.onRef }
      >
        <ParityBackground className={ styles.expanded }>
          <div className={ styles.header }>
            <div className={ styles.title }>
              <ContainerTitle title='Parity Signer: Pending' />
            </div>
            <div className={ styles.actions }>
              <Button
                icon={ <ContentClear /> }
                label='Close'
                onClick={ this.toggleDisplay } />
            </div>
          </div>
          <div className={ styles.content }>
            <Signer />
          </div>
        </ParityBackground>
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

    this.setOpened(!opened);
  }
}

function mapStateToProps (state) {
  const { pending } = state.signer;

  return {
    pending
  };
}

export default connect(
  mapStateToProps,
  null
)(ParityBar);
