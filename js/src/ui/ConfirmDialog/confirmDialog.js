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
import { FormattedMessage } from 'react-intl';

import { nodeOrStringProptype } from '~/util/proptypes';

import Button from '../Button';
import Modal from '../Modal';
import { CancelIcon, CheckIcon } from '../Icons';

import styles from './confirmDialog.css';

const DEFAULT_NO = (
  <FormattedMessage
    id='ui.confirmDialog.no'
    defaultMessage='no'
  />
);
const DEFAULT_YES = (
  <FormattedMessage
    id='ui.confirmDialog.yes'
    defaultMessage='yes'
  />
);

export default class ConfirmDialog extends Component {
  static propTypes = {
    children: PropTypes.node.isRequired,
    className: PropTypes.string,
    iconConfirm: PropTypes.node,
    iconDeny: PropTypes.node,
    labelConfirm: PropTypes.string,
    labelDeny: PropTypes.string,
    onConfirm: PropTypes.func.isRequired,
    onDeny: PropTypes.func.isRequired,
    title: nodeOrStringProptype().isRequired,
    visible: PropTypes.bool.isRequired
  }

  render () {
    const { children, className, title, visible } = this.props;

    return (
      <Modal
        className={ className }
        actions={ this.renderActions() }
        title={ title }
        visible={ visible }
      >
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
        icon={ iconDeny || <CancelIcon /> }
        label={ labelDeny || DEFAULT_NO }
        onClick={ onDeny }
      />,
      <Button
        icon={ iconConfirm || <CheckIcon /> }
        label={ labelConfirm || DEFAULT_YES }
        onClick={ onConfirm }
      />
    ];
  }
}
