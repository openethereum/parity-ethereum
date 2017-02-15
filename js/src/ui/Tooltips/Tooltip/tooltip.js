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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';

import { CancelIcon, DoneIcon, NextIcon } from '~/ui/Icons';
import { nodeOrStringProptype } from '~/util/proptypes';

import { newTooltip, nextTooltip, closeTooltips } from '../actions';

import styles from '../tooltips.css';

let tooltipId = 0;

class Tooltip extends Component {
  static propTypes = {
    className: PropTypes.string,
    currentId: PropTypes.number,
    maxId: PropTypes.number,
    onNewTooltip: PropTypes.func,
    onNextTooltip: PropTypes.func,
    onCloseTooltips: PropTypes.func,
    right: PropTypes.bool,
    text: nodeOrStringProptype(),
    title: nodeOrStringProptype()
  }

  state = {
    id: tooltipId
  }

  componentWillMount () {
    const { onNewTooltip } = this.props;

    onNewTooltip(tooltipId);
    tooltipId++;
  }

  render () {
    const { id } = this.state;
    const { className, currentId, maxId, right, onCloseTooltips, onNextTooltip, text, title } = this.props;

    if (id !== currentId) {
      return null;
    }

    const buttons = id !== maxId
      ? [
        <FlatButton
          icon={ <CancelIcon /> }
          key='skipButton'
          label={
            <FormattedMessage
              id='ui.tooltips.button.skip'
              defaultMessage='Skip'
            />
          }
          onTouchTap={ onCloseTooltips }
        />,
        <FlatButton
          icon={ <NextIcon /> }
          key='nextButton'
          label={
            <FormattedMessage
              id='ui.tooltips.button.next'
              defaultMessage='Next'
            />
          }
          onTouchTap={ onNextTooltip }
        />
      ] : (
        <FlatButton
          icon={ <DoneIcon /> }
          label={
            <FormattedMessage
              id='ui.tooltips.button.done'
              defaultMessage='Done'
            />
          }
          onTouchTap={ onCloseTooltips }
        />
      );

    return (
      <div
        className={
          [
            styles.box,
            right
              ? styles.arrowRight
              : styles.arrowLeft,
            className
          ].join(' ')
        }
      >
        <div className={ styles.title }>
          { title }
        </div>
        <div className={ styles.text }>
          { text }
        </div>
        <div className={ styles.buttons }>
          { buttons }
        </div>
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { currentId, maxId } = state.tooltip;

  return {
    currentId,
    maxId
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onNewTooltip: newTooltip,
    onNextTooltip: nextTooltip,
    onCloseTooltips: closeTooltips
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Tooltip);
