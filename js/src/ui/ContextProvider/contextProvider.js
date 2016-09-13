import { Component, PropTypes } from 'react';

export default class ApiProvider extends Component {
  static propTypes = {
    api: PropTypes.object.isRequired,
    muiTheme: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired,
    children: PropTypes.node.isRequired
  }

  static childContextTypes = {
    api: PropTypes.object,
    muiTheme: PropTypes.object,
    store: PropTypes.object
  }

  render () {
    const { children } = this.props;

    return children;
  }

  getChildContext () {
    const { api, muiTheme, store } = this.props;

    return {
      api,
      muiTheme,
      store
    };
  }
}
