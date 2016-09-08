import React, { Component, PropTypes } from 'react';
import AutoComplete from 'material-ui/AutoComplete';

export default class WrappedAutoComplete extends Component {

  render () {
    return (
      <AutoComplete { ...this.props } />
    );
  }

  static defaultProps = {
    openOnFocus: true,
    filter: (searchText, key) => searchText === '' || key.toLowerCase().indexOf(searchText.toLowerCase()) !== -1
  }

  static propTypes = {
    dataSource: PropTypes.array.isRequired,
    filter: PropTypes.func,
    name: PropTypes.string.isRequired,
    openOnFocus: PropTypes.bool
  }

  static contextTypes = {
    muiTheme: PropTypes.object.isRequired
  }

}
