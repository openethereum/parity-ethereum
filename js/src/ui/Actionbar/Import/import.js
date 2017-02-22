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
import Dropzone from 'react-dropzone';
import { FormattedMessage } from 'react-intl';

import { nodeOrStringProptype } from '~/util/proptypes';

import Button from '../../Button';
import { CancelIcon, DoneIcon, FileUploadIcon } from '../../Icons';
import Portal from '../../Portal';

import styles from './import.css';

const initialState = {
  step: 0,
  show: false,
  validate: false,
  validationBody: null,
  error: false,
  errorText: '',
  content: ''
};

export default class ActionbarImport extends Component {
  static propTypes = {
    className: PropTypes.string,
    onConfirm: PropTypes.func.isRequired,
    renderValidation: PropTypes.func,
    title: nodeOrStringProptype()
  };

  static defaultProps = {
    title: (
      <FormattedMessage
        id='ui.actionbar.import.title'
        defaultMessage='Import from a file'
      />
    )
  };

  state = Object.assign({}, initialState);

  render () {
    const { className } = this.props;

    return (
      <div>
        <Button
          className={ className }
          icon={ <FileUploadIcon /> }
          label={
            <FormattedMessage
              id='ui.actiobar.import.button.import'
              defaultMessage='import'
            />
          }
          onClick={ this.onOpenModal }
        />
        { this.renderModal() }
      </div>
    );
  }

  renderModal () {
    const { title, renderValidation } = this.props;
    const { show, step, error } = this.state;

    if (!show) {
      return null;
    }

    const steps = typeof renderValidation === 'function'
      ? [
        <FormattedMessage
          id='ui.actiobar.import.step.select'
          defaultMessage='select a file'
        />,
        error
          ? (
            <FormattedMessage
              id='ui.actiobar.import.step.error'
              defaultMessage='error'
            />
          )
          : (
            <FormattedMessage
              id='ui.actiobar.import.step.validate'
              defaultMessage='validate'
            />)
      ]
      : null;

    return (
      <Portal
        activeStep={ step }
        buttons={ this.renderActions() }
        onClose={ this.onCloseModal }
        open
        steps={ steps }
        title={ title }
      >
        { this.renderBody() }
      </Portal>
    );
  }

  renderActions () {
    const { validate, error } = this.state;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='ui.actiobar.import.button.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.onCloseModal }
      />
    );

    if (error) {
      return [ cancelBtn ];
    }

    if (validate) {
      const confirmBtn = (
        <Button
          icon={ <DoneIcon /> }
          key='confirm'
          label={
            <FormattedMessage
              id='ui.actiobar.import.button.confirm'
              defaultMessage='Confirm'
            />
          }
          onClick={ this.onConfirm }
        />
      );

      return [ cancelBtn, confirmBtn ];
    }

    return [ cancelBtn ];
  }

  renderBody () {
    const { validate, errorText, error } = this.state;

    if (error) {
      return (
        <div>
          <p>
            <FormattedMessage
              id='ui.actiobar.import.error'
              defaultMessage='An error occured: {errorText}'
              values={ {
                errorText
              } }
            />
          </p>
        </div>
      );
    }

    if (validate) {
      return this.renderValidation();
    }

    return this.renderFileSelect();
  }

  renderFileSelect () {
    return (
      <div>
        <Dropzone
          onDrop={ this.onDrop }
          multiple={ false }
          className={ styles.importZone }
        >
          <div>
            <FormattedMessage
              id='ui.actiobar.import.dropzone'
              defaultMessage='Drop a file here, or click to select a file to upload.'
            />
          </div>
        </Dropzone>
      </div>
    );
  }

  renderValidation () {
    const { validationBody } = this.state;

    return (
      <div>
        <p className={ styles.desc }>
          <FormattedMessage
            id='ui.actiobar.import.confirm'
            defaultMessage='Confirm that this is what was intended to import.'
          />
        </p>
        <div>
          { validationBody }
        </div>
      </div>
    );
  }

  onDrop = (files) => {
    const { renderValidation } = this.props;

    const file = files[0];
    const reader = new FileReader();

    reader.onload = (e) => {
      const content = e.target.result;

      if (typeof renderValidation !== 'function') {
        this.props.onConfirm(content);
        return this.onCloseModal();
      }

      const validationBody = renderValidation(content);

      if (validationBody && validationBody.error) {
        return this.setState({
          step: 1,
          error: true,
          errorText: validationBody.error
        });
      }

      this.setState({
        step: 1,
        validate: true,
        validationBody,
        content
      });
    };

    reader.readAsText(file);
  }

  onConfirm = () => {
    const { content } = this.state;

    this.props.onConfirm(content);
    return this.onCloseModal();
  }

  onOpenModal = () => {
    this.setState({ show: true });
  }

  onCloseModal = () => {
    this.setState(initialState);
  }
}
