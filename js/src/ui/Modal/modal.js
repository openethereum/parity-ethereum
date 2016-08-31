import React, { Component, PropTypes } from 'react';

import { Dialog } from 'material-ui';

import ModalSteps from './ModalSteps';

const ACTIONS_STYLE = { borderStyle: 'none' };
const TITLE_STYLE = { borderStyle: 'none' };
const DIALOG_STYLE = { paddingTop: '1px' };
const CONTENT_STYLE = { transform: 'translate(0px, 0px)' };

export default class Modal extends Component {
  static propTypes = {
    actions: PropTypes.node,
    children: PropTypes.node,
    current: PropTypes.number,
    scroll: PropTypes.bool,
    steps: PropTypes.array,
    title: React.PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ]),
    visible: PropTypes.bool.isRequired
  }

  render () {
    const { actions, current, children, scroll, steps, title, visible } = this.props;
    let header = title;

    if (steps) {
      header = (
        <ModalSteps
          current={ current }
          steps={ steps }
          title={ title } />
      );
    }

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
