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

import { Dialog } from 'material-ui';

import Title from './Title';

const ACTIONS_STYLE = { borderStyle: 'none' };
const TITLE_STYLE = { borderStyle: 'none' };
const DIALOG_STYLE = { paddingTop: '1px' };
const CONTENT_STYLE = { transform: 'translate(0px, 0px)' };

import styles from './modal.css';

export default class Modal extends Component {
  static propTypes = {
    actions: PropTypes.node,
    children: PropTypes.node,
    current: PropTypes.number,
    waiting: PropTypes.array,
    scroll: PropTypes.bool,
    steps: PropTypes.array,
    title: React.PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ]),
    visible: PropTypes.bool.isRequired
  }

  render () {
    const { actions, current, children, scroll, steps, waiting, title, visible } = this.props;
    const header = (
      <Title
        current={ current }
        waiting={ waiting }
        steps={ steps }
        title={ title } />
    );

    return (
      <Dialog
        actions={ actions }
        actionsContainerStyle={ ACTIONS_STYLE }
        autoDetectWindowHeight={ false }
        autoScrollBodyContent={ !!scroll }
        contentStyle={ CONTENT_STYLE }
        modal
        open={ visible }
        repositionOnUpdate={ false }
        style={ DIALOG_STYLE }
        title={ header }
        titleStyle={ TITLE_STYLE }>
        { children }
      </Dialog>
    );
  }
}
