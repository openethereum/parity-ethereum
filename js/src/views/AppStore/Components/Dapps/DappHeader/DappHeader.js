/* @flow */
import React, { Component } from 'react';

/** Stylesheets **/
import styles from './DappHeader.css';

// type Props = {|
//   history: Object
// |}

// type State = {|
//
// |}

class DappHeader extends Component {
  // props: Props;
  // state: State = {
  // };

  backClick = () => {
    const { goBack } = this.props.history;

    goBack();
  }

  render() {

    return (
      <div className={styles.DappHeader}>

        <div className={styles.backButton}>
          <svg onClick={this.backClick} x="0px" y="0px" width="63px" height="50px" viewBox="0 0 63 50">
            <g>
            <g>
              <linearGradient id="SVGID_x_" gradientUnits="userSpaceOnUse" x1="32.3335" y1="8.5425" x2="32.3335" y2="41.4575">
                <stop  offset="0" style={{stopColor: '#828282'}}/>
                <stop  offset="1" style={{stopColor: '#353535'}}/>
              </linearGradient>
              <path fillRule="evenodd" clipRule="evenodd" fill="url(#SVGID_x_)" d="M32.344,19.498c0.213,0,0.381,0,0.549,0
                c6.143,0,12.286,0,18.429,0c1.828,0,2.967,1.139,2.968,2.965c0.001,1.714,0.003,3.428-0.001,5.143
                c-0.003,1.679-1.18,2.862-2.855,2.863c-6.171,0.002-12.343,0.001-18.515,0.001c-0.17,0-0.341,0-0.574,0c0,0.169,0,0.319,0,0.47
                c0,2.514-0.006,5.028,0.003,7.543c0.004,1.206-0.453,2.157-1.558,2.688c-1.142,0.548-2.187,0.274-3.137-0.52
                c-3.256-2.719-6.517-5.432-9.779-8.144c-1.933-1.608-3.872-3.208-5.802-4.818c-0.35-0.292-0.698-0.596-0.997-0.938
                c-0.926-1.062-0.943-2.505,0.016-3.541c0.579-0.625,1.258-1.161,1.916-1.708c4.872-4.06,9.766-8.094,14.618-12.177
                c1.777-1.496,4.04-0.698,4.579,1.123c0.095,0.323,0.134,0.673,0.135,1.011C32.349,14.116,32.344,16.773,32.344,19.498z"/>
            </g>
            <defs>
              <filter id="Adobe_OpacityMaskFilter" filterUnits="userSpaceOnUse" x="10.376" y="8.542" width="43.915" height="32.915">

                  <feColorMatrix  type="matrix" values="-1 0 0 0 1  0 -1 0 0 1  0 0 -1 0 1  0 0 0 1 0" colorInterpolationFilters="sRGB" result="source"/>
              </filter>
            </defs>
            <mask maskUnits="userSpaceOnUse" x="10.376" y="8.542" width="43.915" height="32.915" id="SVGID_2_">
              <g filter="url(#Adobe_OpacityMaskFilter)">

                  <image overflow="visible" width="200" height="154" transform="matrix(0.24 0 0 0.24 8.3765 6.542)">
                </image>
              </g>
            </mask>
            <g opacity="1" mask="url(#SVGID_2_)">
              <path fillRule="evenodd" clipRule="evenodd" fill="#2B2B2B" d="M32.344,19.498c0.213,0,0.381,0,0.549,0
                c6.143,0,12.286,0,18.429,0c1.828,0,2.967,1.139,2.968,2.965c0.001,1.714,0.003,3.428-0.001,5.143
                c-0.003,1.679-1.18,2.862-2.855,2.863c-6.171,0.002-12.343,0.001-18.515,0.001c-0.17,0-0.341,0-0.574,0c0,0.169,0,0.319,0,0.47
                c0,2.514-0.006,5.028,0.003,7.543c0.004,1.206-0.453,2.157-1.558,2.688c-1.142,0.548-2.187,0.274-3.137-0.52
                c-3.256-2.719-6.517-5.432-9.779-8.144c-1.933-1.608-3.872-3.208-5.802-4.818c-0.35-0.292-0.698-0.596-0.997-0.938
                c-0.926-1.062-0.943-2.505,0.016-3.541c0.579-0.625,1.258-1.161,1.916-1.708c4.872-4.06,9.766-8.094,14.618-12.177
                c1.777-1.496,4.04-0.698,4.579,1.123c0.095,0.323,0.134,0.673,0.135,1.011C32.349,14.116,32.344,16.773,32.344,19.498z"/>
            </g>
            </g>
          </svg>
          <div id={styles.backName}>
            Featured
          </div>
        </div>

        <div className={styles.appName}>
          App Name
        </div>

        <div id={styles.dappHeaderSearch}>
          <input type="text" id={styles.dappHeaderInput}/>
        </div>

      </div>
    );
  }
}

export default DappHeader;
