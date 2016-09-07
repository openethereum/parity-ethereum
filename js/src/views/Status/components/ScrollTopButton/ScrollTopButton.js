
import React, { Component } from 'react';
import IconButton from 'material-ui/IconButton';
import ArrowUpwardIcon from 'material-ui/svg-icons/navigation/arrow-upward';

import { scrollTo } from './util';
import styles from './ScrollTopButton.css';

const scrollTopThreshold = 600;

export default class ScrollTopButton extends Component {

  state = {}

  componentDidMount () {
    window.addEventListener('scroll', this.handleScroll);
  }

  componentWillUnmount () {
    window.removeEventListener('scroll', this.handleScroll);
  }

  _scrollToTop () {
    scrollTo(document.body, 0, 500);
  }

  render () {
    let hiddenClass = !this.state.showScrollButton ? styles.hidden : '';

    return (
      <IconButton
        className={ `${styles.scrollButton} ${hiddenClass}` }
        onClick={ this._scrollToTop }>
        <ArrowUpwardIcon />
      </IconButton>
    );
  }

  handleScroll = event => {
    let { scrollTop } = event.srcElement.body;
    let { showScrollButton } = this.state;

    if (!showScrollButton && scrollTop > scrollTopThreshold) {
      this.setState({
        showScrollButton: true
      });
    }

    if (showScrollButton && scrollTop < scrollTopThreshold) {
      this.setState({
        showScrollButton: false
      });
    }
  }

}
