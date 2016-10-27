import React, { Component, PropTypes } from 'react';
import ActionDone from 'material-ui/svg-icons/action/done';
import ContentClear from 'material-ui/svg-icons/content/clear';

import Button from '../Button';
import Modal from '../Modal';

import styles from './confirmDialog.css';

export default class ConfirmDialog extends Component {
  static propTypes = {
    children: PropTypes.node.isRequired,
    className: PropTypes.string,
    iconConfirm: PropTypes.node,
    iconDeny: PropTypes.node,
    labelConfirm: PropTypes.string,
    labelDeny: PropTypes.string,
    title: PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ]).isRequired,
    visible: PropTypes.bool.isRequired,
    onConfirm: PropTypes.func.isRequired,
    onDeny: PropTypes.func.isRequired
  }

  render () {
    const { children, className, title, visible } = this.props;

    return (
      <Modal
        className={ className }
        actions={ this.renderActions() }
        title={ title }
        visible={ visible }>
        <div className={ styles.body }>
          { children }
        </div>
      </Modal>
    );
  }

  renderActions () {
    const { iconConfirm, iconDeny, labelConfirm, labelDeny, onConfirm, onDeny } = this.props;

    return [
      <Button
        label={ labelDeny || 'no' }
        icon={ iconDeny || <ContentClear /> }
        onClick={ onDeny } />,
      <Button
        label={ labelConfirm || 'yes' }
        icon={ iconConfirm || <ActionDone /> }
        onClick={ onConfirm } />
    ];
  }
}
