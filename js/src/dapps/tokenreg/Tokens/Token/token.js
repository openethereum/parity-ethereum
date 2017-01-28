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
import Paper from 'material-ui/Paper';
import { RaisedButton, SelectField, MenuItem } from 'material-ui';

import FindIcon from 'material-ui/svg-icons/action/find-in-page';
import DeleteIcon from 'material-ui/svg-icons/action/delete';

import Loading from '../../Loading';
import Chip from '../../Chip';
import AddMeta from './add-meta';

import styles from './token.css';

import { metaDataKeys } from '../../constants';

import { api } from '../../parity';
import { parityNode } from '../../../../environment';

export default class Token extends Component {
  static propTypes = {
    handleMetaLookup: PropTypes.func.isRequired,
    address: PropTypes.string.isRequired,
    name: PropTypes.string.isRequired,
    index: PropTypes.number.isRequired,
    owner: PropTypes.string.isRequired,

    handleAddMeta: PropTypes.func,
    handleUnregister: PropTypes.func,

    tla: PropTypes.string,
    base: PropTypes.number,
    totalSupply: PropTypes.number,
    meta: PropTypes.object,
    isMetaLoading: PropTypes.bool,
    ownerAccountInfo: PropTypes.shape({
      name: PropTypes.string,
      meta: PropTypes.object
    }),
    metaPending: PropTypes.bool,
    metaMined: PropTypes.bool,
    isLoading: PropTypes.bool,
    isPending: PropTypes.bool,
    isTokenOwner: PropTypes.bool.isRequired,
    isContractOwner: PropTypes.bool,

    fullWidth: PropTypes.bool
  };

  static defaultProps = {
    isContractOwner: false
  };

  state = {
    metaKeyIndex: 0,
    showMeta: false
  };

  shouldComponentUpdate (nextProps) {
    if (nextProps.isLoading && this.props.isLoading) {
      return false;
    }

    return true;
  }

  render () {
    const { isLoading, fullWidth } = this.props;

    if (isLoading) {
      return (
        <div className={ [ styles.token, styles.loading ].join(' ') }>
          <Loading size={ 1 } />
        </div>
      );
    }

    if (fullWidth) {
      return (<div className={ styles['full-width'] }>
        { this.renderContent() }
      </div>);
    }

    return (<div>
      <Paper zDepth={ 1 } className={ styles.token } style={ {
        backgroundColor: 'none'
      } }
      >
        <div className={ styles['token-bg'] } />
        { this.renderContent() }
      </Paper>
    </div>);
  }

  renderContent () {
    const { address, tla, base, name, meta, owner, totalSupply } = this.props;

    return (<div className={ styles['token-container'] }>
      { this.renderIsPending() }

      <div className={ styles['token-content'] }>
        <div className={ styles.title }>{ tla }</div>
        <div className={ styles.name }>"{ name }"</div>

        { this.renderBase(base) }
        { this.renderTotalSupply(totalSupply, base, tla) }
        { this.renderAddress(address) }
        { this.renderOwner(owner) }
      </div>

      <div className={ styles['token-meta'] }>
        <div className={ styles['meta-form'] }>
          <SelectField
            floatingLabelText='Choose the meta-data to look-up'
            fullWidth
            value={ this.state.metaKeyIndex }
            onChange={ this.onMetaKeyChange }
          >

            { this.renderMetaKeyItems() }

          </SelectField>

          <RaisedButton
            label='Lookup'
            icon={ <FindIcon /> }
            primary
            fullWidth
            onTouchTap={ this.onMetaLookup }
          />
        </div>

        { this.renderMeta(meta) }
        { this.renderAddMeta() }
        { this.renderUnregister() }
      </div>

      { this.renderMetaPending() }
      { this.renderMetaMined() }
    </div>);
  }

  renderMetaKeyItems () {
    return metaDataKeys.map((key, index) => (
      <MenuItem
        value={ index }
        key={ index }
        label={ key.label } primaryText={ key.label }
      />
    ));
  }

  renderBase (base) {
    if (!base || base < 0) {
      return null;
    }

    return (
      <Chip
        value={ Math.log10(base).toString() }
        label='Decimals'
      />
    );
  }

  renderAddress (address) {
    if (!address) {
      return null;
    }

    return (
      <Chip
        isAddress
        value={ address }
        label='Address'
      />
    );
  }

  renderTotalSupply (totalSupply, base, tla) {
    const balance = Math.round((totalSupply / base) * 100) / 100;

    return (
      <Chip
        value={ `${balance.toString()} ${tla}` }
        label='Total'
      />
    );
  }

  renderOwner (owner) {
    if (!owner) {
      return null;
    }

    const ownerInfo = this.props.ownerAccountInfo;

    const displayValue = (ownerInfo && ownerInfo.name)
      ? ownerInfo.name
      : owner;

    return (
      <Chip
        isAddress
        displayValue={ displayValue }
        value={ owner }
        label='Owner'
      />
    );
  }

  renderIsPending () {
    const { isPending } = this.props;

    if (!isPending) {
      return null;
    }

    return (
      <div className={ styles.pending } />
    );
  }

  renderAddMeta () {
    if (!this.props.isTokenOwner) {
      return null;
    }

    return (
      <AddMeta
        handleAddMeta={ this.props.handleAddMeta }
        isTokenOwner={ this.props.isTokenOwner }
        index={ this.props.index }
      />
    );
  }

  renderUnregister () {
    if (!this.props.isContractOwner) {
      return null;
    }

    return (
      <RaisedButton
        className={ styles.unregister }
        label='Unregister'
        icon={ <DeleteIcon /> }
        secondary
        fullWidth
        onTouchTap={ this.onUnregister }
      />
    );
  }

  renderMeta (meta) {
    const { isMetaLoading } = this.props;
    const { showMeta } = this.state;

    if (!showMeta) {
      return null;
    }

    if (isMetaLoading) {
      return (
        <div>
          <Loading size={ 0.5 } />
        </div>
      );
    }

    if (!meta) {
      return null;
    }

    const metaData = metaDataKeys.find(m => m.value === meta.query);

    if (!meta.value) {
      return (
        <div>
          <p className={ styles['meta-query'] }>
            No <span className={ styles['meta-key'] }>
              { metaData.label.toLowerCase() }
            </span> meta-data...
          </p>
        </div>
      );
    }

    if (meta.query === 'IMG') {
      const imageHash = meta.value.replace(/^0x/, '');

      return (
        <div>
          <p className={ styles['meta-query'] }>
            <span className={ styles['meta-key'] }>
              { metaData.label }
            </span> meta-data:
          </p>
          <div className={ styles['meta-image'] }>
            <img src={ `${parityNode}/api/content/${imageHash}/` } />
          </div>
        </div>
      );
    }

    if (meta.query === 'A') {
      const address = meta.value.slice(0, 42);

      return (
        <div>
          <p className={ styles['meta-query'] }>
            <span className={ styles['meta-key'] }>
              { metaData.label }
            </span> meta-data:
          </p>
          <p className={ styles['meta-value'] }>
            { api.util.toChecksumAddress(address) }
          </p>
        </div>
      );
    }

    return (
      <div>
        <p className={ styles['meta-query'] }>
          <span className={ styles['meta-key'] }>
            { metaData.label }
          </span> meta-data:
        </p>
        <p className={ styles['meta-value'] }>{ meta.value }</p>
      </div>
    );
  }

  renderMetaPending () {
    const isMetaPending = this.props.metaPending;

    if (!isMetaPending) {
      return null;
    }

    return (
      <div>
        <p className={ styles['meta-info'] }>
          Meta-Data pending...
        </p>
      </div>
    );
  }

  renderMetaMined () {
    const isMetaMined = this.props.metaMined;

    if (!isMetaMined) {
      return null;
    }

    return (
      <div>
        <p className={ styles['meta-info'] }>
          Meta-Data saved on the blockchain!
        </p>
      </div>
    );
  }

  onUnregister = () => {
    const index = this.props.index;

    this.props.handleUnregister(index);
  }

  onMetaLookup = () => {
    const keyIndex = this.state.metaKeyIndex;
    const key = metaDataKeys[keyIndex].value;
    const index = this.props.index;

    this.setState({ showMeta: true });
    this.props.handleMetaLookup(index, key);
  }

  onMetaKeyChange = (event, metaKeyIndex) => {
    this.setState({ metaKeyIndex });
  }
}
