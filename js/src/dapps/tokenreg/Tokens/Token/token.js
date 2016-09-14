import React, { Component, PropTypes } from 'react';
import Paper from 'material-ui/Paper';
import { RaisedButton, TextField } from 'material-ui';
import FindIcon from 'material-ui/svg-icons/action/find-in-page';

import Loading from '../../Loading';
import Chip from '../../Chip';

import styles from './token.css';

export default class Token extends Component {
  static propTypes = {
    handleMetaLookup: PropTypes.func,
    isLoading: PropTypes.bool,
    address: PropTypes.string,
    tla: PropTypes.string,
    name: PropTypes.string,
    base: PropTypes.number,
    index: PropTypes.number
  };

  state = {
    metaQuery: ''
  };

  render () {
    const { isLoading, address, tla, base, name, meta } = this.props;

    if (isLoading) {
      return (
        <div className={ styles.token }>
          <Loading size={1} />
        </div>
      );
    }

    return (
      <Paper zDepth={2} className={ styles.token }>
        <div className={ styles.title }>{ tla }</div>
        <div className={ styles.name }>"{ name }"</div>

        <Chip
          value={base.toString()}
          label="Base" />

        <Chip
          isAddress={true}
          value={address}
          label="Address" />

        <div className={ styles.metaForm }>
          <TextField
            autoComplete="off"
            floatingLabelFixed
            fullWidth
            floatingLabelText="Meta Key"
            hintText="The key of the meta-data to lookup"
            value={ this.state.metaQuery }
            onChange={ this.onMetaQueryChange } />

          <RaisedButton
            label='Lookup'
            icon={ <FindIcon /> }
            primary
            fullWidth
            onTouchTap={ this.onMetaLookup } />
        </div>

        { this.renderMeta(meta) }
      </Paper>
    );
  }

  renderMeta(meta) {
    if (!meta) return;

    return (<div>
      <p className={ styles['meta-query'] }>{meta.query}</p>
      <p className={ styles['meta-value'] }>{meta.value}</p>
    </div>);
  }

  onMetaLookup = () => {
    let query = this.state.metaQuery;
    let index = this.props.index;

    this.props.handleMetaLookup(index, query);
  }

  onMetaQueryChange = (event, metaQuery) => {
    this.setState({ metaQuery });
  }
}
