#!/usr/bin/env bash


GET_DEPS_URL=http://get-deps.ethcore.io
#PARITY_DEB_URL=https://github.com/ethcore/parity/releases/download/beta-0.9/parity_0.9.0-0_amd64.deb
PARITY_DEB_URL=https://github.com/jesuscript/scripts/raw/master/parity_0.9.0-0_amd64.deb

function run_installer()
{
  ####### Init vars
  
  HOMEBREW_PREFIX=/usr/local
  HOMEBREW_CACHE=/Library/Caches/Homebrew
  HOMEBREW_REPO=https://github.com/Homebrew/homebrew
  OSX_REQUIERED_VERSION="10.7.0"
  
  declare OS_TYPE
  declare OSX_VERSION
  declare GIT_PATH
  declare RUBY_PATH
  declare BREW_PATH
  declare INSTALL_FILES=""

  errorMessages=""
  isOsVersion=false
  isGit=false
  isRuby=false
  isBrew=false
  canContinue=true
  depCount=0
  depFound=0

  
  ####### Setup colors

  red=`tput setaf 1`
  green=`tput setaf 2`
  yellow=`tput setaf 3`
  blue=`tput setaf 4`
  magenta=`tput setaf 5`
  cyan=`tput setaf 6`
  white=`tput setaf 7`
  b=`tput bold`
  u=`tput sgr 0 1`
  ul=`tput smul`
  xl=`tput rmul`
  stou=`tput smso`
  xtou=`tput rmso`
  dim=`tput dim`
  reverse=`tput rev`
  reset=`tput sgr0`


  function head() {
    echo "${blue}${b}==>${white} $1${reset}"
  }

  function info() {
    echo "${blue}${b}==>${reset} $1"
  }

  function successHeading() {
    echo "${green}${b}==> $1${reset}"
  }

  function success() {
    echo "${green}${b}==>${reset}${green} $1${reset}"
  }

  function error() {
    echo "${red}==> ${u}${b}${red}$1${reset}"
  }

  function smallError() {
    echo "${red}==>${reset} $1"
  }

  function green() {
    echo "${green}$1${reset}"
  }

  function red() {
    echo "${red}$1${reset}"
  }

  function check() {
    echo "${green}${bold} ✓${reset}  $1${reset}"
  }

  function uncheck() {
    echo "${red}${bold} ✘${reset}  $1${reset}"
  }



  ####### Setup methods

  function wait_for_user() {
    while :
    do
      read -p "${blue}==>${reset} $1 [Y/n] " imp
      case $imp in
        [yY] ) return 0; break ;;
        '' ) echo; break ;;
        [nN] ) return 1 ;;
        * ) echo "Unrecognized option provided. Please provide either 'Y' or 'N'";
      esac
    done
  }

  function prompt_for_input() {
    while :
    do
      read -p "$1 " imp
      echo $imp
      return
    done
  }
  
  function exe() {
    echo "\$ $@"; "$@"
  }
  
  function detectOS() {
    if [[ "$OSTYPE" == "linux-gnu" ]]
    then
      OS_TYPE="linux"
      get_linux_dependencies
    elif [[ "$OSTYPE" == "darwin"* ]]
    then
      OS_TYPE="osx"
      get_osx_dependencies
    else
      OS_TYPE="win"
      abortInstall "${red}==>${reset} ${b}OS not supported:${reset} parity one-liner currently support OS X and Linux.\nFor instructions on installing parity on other platforms please visit ${u}${blue}http://ethcore.io/${reset}"
    fi

    echo

    if [[ $depCount == $depFound ]]
    then
      green "Found all dependencies ($depFound/$depCount)"
    else
      if [[ $canContinue == true ]]
      then
        red "Some dependencies are missing ($depFound/$depCount)"
      elif [[ $canContinue == false && $depFound == 0 ]]
      then
        red "All dependencies are missing and cannot be auto-installed ($depFound/$depCount)"
        abortInstall "$errorMessages";
      elif [[ $canContinue == false ]]
      then
        red "Some dependencies which cannot be auto-installed are missing ($depFound/$depCount)"
        abortInstall "$errorMessages";
      fi
    fi
  }

  function linux_version()
  {
    source /etc/lsb-release
    
    if [[ $DISTRIB_ID == "Ubuntu" ]]; then
      if [[ $DISTRIB_RELEASE == "14.04" ]]; then
        check "Ubuntu-14.04"
        isUbuntu1404=true
      else
        check "Ubuntu, but not 14.04"
        isUbuntu1404=false
      fi
    else
      check "Ubuntu not found"
      isUbuntu1404=false
    fi
  }

  function get_linux_dependencies()
  {
    linux_version

    find_multirust
    find_rocksdb

    find_curl
    find_git
    find_make
    find_gcc

    find_apt
  }

  function find_rocksdb()
  {
    depCount=$((depCount+1))
    if [[ $(ldconfig -v 2>/dev/null | grep rocksdb | wc -l) == 1 ]]; then
      depFound=$((depFound+1))
      check "apt-get"
      isRocksDB=true
    else
      uncheck "librocksdb is missing"
      isRocksDB=false
      INSTALL_FILES+="${blue}${dim}==>${reset}\tlibrocksdb\n"
    fi
  }

  function find_multirust()
  {
    depCount=$((depCount+2))
    MULTIRUST_PATH=`which multirust 2>/dev/null`
    if [[ -f $MULTIRUST_PATH ]]; then
      depFound=$((depFound+1))
      check "multirust"
      isMultirust=true
      if [[ $(multirust show-default 2>/dev/null | grep nightly | wc -l) == 4 ]]; then
        depFound=$((depFound+1))
        check "rust nightly"
        isMultirustNightly=true
      else
        uncheck "rust is not nightly"
        isMultirustNightly=false
        INSTALL_FILES+="${blue}${dim}==>${reset}\tmultirust -> rust nightly\n"
      fi
    else
      uncheck "multirust is missing"
      uncheck "rust nightly is missing"
      isMultirust=false
      isMultirustNightly=false
      INSTALL_FILES+="${blue}${dim}==>${reset}\tmultirust\n"
    fi
  }

  function find_apt()
  {
    depCount=$((depCount+1))

    APT_PATH=`which apt-get 2>/dev/null`

    if [[ -f $APT_PATH ]]
    then
      depFound=$((depFound+1))
      check "apt-get"
      isApt=true
    else
      uncheck "apt-get is missing"
      isApt=false

      if [[ $isGCC == false || $isGit == false || $isMake == false || $isCurl == false ]]; then
        canContinue=false
        errorMessages+="${red}==>${reset} ${b}Couldn't find apt-get:${reset} We can only use apt-get in order to grab our dependencies.\n"
        errorMessages+="    Please switch to a distribution such as Debian or Ubuntu or manually install the missing packages.\n"
      fi
    fi
  }

  function find_gcc()
  {
    depCount=$((depCount+1))
    GCC_PATH=`which g++ 2>/dev/null`

    if [[ -f $GCC_PATH ]]
    then
      depFound=$((depFound+1))
      check "g++"
      isGCC=true
    else
      uncheck "g++ is missing"
      isGCC=false
      INSTALL_FILES+="${blue}${dim}==>${reset}\tg++\n"
    fi
  }

  function find_git()
  {
    depCount=$((depCount+1))
    GIT_PATH=`which git 2>/dev/null`

    if [[ -f $GIT_PATH ]]
    then
      depFound=$((depFound+1))
      check "git"
      isGit=true
    else
      uncheck "git is missing"
      isGit=false
      INSTALL_FILES+="${blue}${dim}==>${reset}\tgit\n"
    fi
  }

  function find_make()
  {
    depCount=$((depCount+1))
    MAKE_PATH=`which make 2>/dev/null`

    if [[ -f $MAKE_PATH ]]
    then
      depFound=$((depFound+1))
      check "make"
      isMake=true
    else
      uncheck "make is missing"
      isMake=false
      INSTALL_FILES+="${blue}${dim}==>${reset}\tmake\n"
    fi
  }

  function find_curl()
  {
    depCount=$((depCount+1))
    CURL_PATH=`which curl 2>/dev/null`

    if [[ -f $CURL_PATH ]]
    then
      depFound=$((depFound+1))
      check "curl"
      isCurl=true
    else
      uncheck "curl is missing"
      isCurl=false
      INSTALL_FILES+="${blue}${dim}==>${reset}\tcurl\n"
    fi
  }

  function ubuntu1404_rocksdb_installer()
  {
    sudo apt-get update -qq
    sudo apt-get install -qq -y software-properties-common
    sudo apt-add-repository -y ppa:giskou/librocksdb
    sudo apt-get -f -y install
    sudo apt-get update -qq
    sudo apt-get install -qq -y librocksdb
  }

  function linux_rocksdb_installer()
  {
    if [[ $isUbuntu1404 == true ]]; then
      ubuntu1404_rocksdb_installer
    else
      oldpwd=`pwd`
      cd /tmp
      exe git clone --branch v4.1 --depth=1 https://github.com/facebook/rocksdb.git
      cd rocksdb
      exe make shared_lib
      sudo cp -a librocksdb.so* /usr/lib
      sudo ldconfig
      cd /tmp
      rm -rf /tmp/rocksdb
      cd $oldpwd
    fi
  }



  function verify_installation()
  {
    ETH_PATH=`which parity 2>/dev/null`

    if [[ -f $ETH_PATH ]]
    then
      success "Parity has been installed"
    else
      error "Parity is missing"
      abortInstall
    fi
  }
	
  function verify_dep_installation()
  {
    info "Verifying installation"

    if [[ $OS_TYPE == "linux" ]]; then
      find_curl
      find_git
      find_make
      find_gcc
      find_rocksdb
      find_multirust

      if [[ $isCurl == false || $isGit == false || $isMake == false || $isGCC == false || $isRocksDB == false || $isMultirustNightly == false ]]; then
        abortInstall
      fi
    fi
  }
	
  function linux_deps_installer()
  {
    if [[ $isGCC == false || $isGit == false || $isMake == false || $isCurl == false ]]; then
      info "Installing build dependencies..."
      sudo apt-get update -qq
      if [[ $isGit == false ]]; then
        sudo apt-get install -q -y git
      fi
      if [[ $isGCC == false ]]; then
        sudo apt-get install -q -y g++ gcc
      fi
      if [[ $isMake == false ]]; then
        sudo apt-get install -q -y make
      fi
      if [[ $isCurl == false ]]; then
        sudo apt-get install -q -y curl
      fi
      echo
    fi

    if [[ $isRocksDB == false ]]; then
      info "Installing rocksdb..."
      linux_rocksdb_installer
      echo
    fi

    if [[ $isMultirust == false ]]; then
      info "Installing multirust..."
      curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sudo sh -s -- --yes
      echo
    fi

    if [[ $isMultirustNightly == false ]]; then
      info "Installing rust nightly..."
      sudo multirust update nightly
      sudo multirust default nightly
      echo
		fi

  }
	
  function linux_installer()
  {
		linux_deps_installer
		verify_dep_installation

		info "Installing parity"
		file=/tmp/parity.deb

		wget $PARITY_DEB_URL -qO $file
		sudo dpkg -i $file
		rm $file
  }

  function osx_installer()
  {
    info "Adding ethcore repository"
    brew tap ethcore/ethcore https://github.com/ethcore/homebrew-ethcore.git
    echo

    info "Updating brew"
    brew update
    echo

    info "Installing parity"
    if [[ $isEth == true ]]
    then
      brew reinstall parity
    else
      brew install parity
      brew linkapps parity
    fi
    echo
  }
	
  function install()
  {
    echo
    head "Installing Parity build dependencies"

    if [[ $OS_TYPE == "osx" ]]
    then
      osx_installer
    elif [[ $OS_TYPE == "linux" ]]
    then
      linux_installer
    fi

		verify_installation
  }

  
  function install_netstats()
  {
    echo "Installing netstats"

    secret=$(prompt_for_input "Please enter the netstats secret:")
    instance_name=$(prompt_for_input "Please enter your instance name:")
    contact_details=$(prompt_for_input "Please enter your contact details (optional):")

    curl -sL https://deb.nodesource.com/setup_0.12 | bash -
    sudo apt-get update
    # install ethereum & install dependencies
    sudo apt-get install -y -qq build-essential git unzip wget nodejs ntp cloud-utils

    # add node symlink if it doesn't exist
    [[ ! -f /usr/bin/node ]] && sudo ln -s /usr/bin/nodejs /usr/bin/node

    # set up time update cronjob
    sudo bash -c "cat > /etc/cron.hourly/ntpdate << EOF
    #!/bin/sh
    pm2 flush
    sudo service ntp stop
    sudo ntpdate -s ntp.ubuntu.com
    sudo service ntp start
    EOF"

    sudo chmod 755 /etc/cron.hourly/ntpdate

		cd $HOME

    [ ! -d "www" ] && git clone https://github.com/cubedro/eth-net-intelligence-api netstats
    oldpwd= $(pwd)
    cd netstats
    git pull
    git checkout 95d595258239a0fdf56b97dedcfb2be62f6170e6

    sudo npm install
    sudo npm install pm2 -g

    cat > app.json << EOL
[
  {
    "name"              : "node-app",
    "script"            : "app.js",
    "log_date_format"   : "YYYY-MM-DD HH:mm Z",
    "merge_logs"        : false,
    "watch"             : false,
    "max_restarts"      : 10,
    "exec_interpreter"  : "node",
    "exec_mode"         : "fork_mode",
    "env":
    {
      "NODE_ENV"        : "production",
      "RPC_HOST"        : "localhost",
      "RPC_PORT"        : "8545",
      "LISTENING_PORT"  : "30303",
      "INSTANCE_NAME"   : "${instance_name}",
      "CONTACT_DETAILS" : "${contact_details}",
      "WS_SERVER"       : "wss://rpc.ethstats.net",
      "WS_SECRET"       : "${secret}",
      "VERBOSITY"       : 2
    
    }
  }
]
EOL

    pm2 startOrRestart app.json
    cd $oldpwd
  }
  

  function abortInstall()
  {
    echo
    error "Installation failed"
    echo -e "$1"
    echo
    exit 0
  }

  function finish()
  {
    echo
    successHeading "All done"
    #   head "Next steps"
    #   info "Run ${cyan}\`\`${reset} to get started.${reset}"
    echo
    exit 0
  }

  head "Checking OS dependencies"
  detectOS

  if [[ $INSTALL_FILES != "" ]]; then
    echo
    head "In addition to the Parity build dependencies, this script will install:"
    printf "$INSTALL_FILES"
    echo
  fi

  #DEBUG

  
  head "${b}OK,${reset} let's install Parity now!"
  echo
  echo
  install

  if [[ $OS_TYPE == "linux" ]]
  then
    echo "Netstats:"
    head "Would you like to install and configure a netstats client?"
    if wait_for_user "${b}OK,${reset} let's go!"
    then
      install_netstats
    fi
  fi


  # Display goodbye message
  finish
}

run_installer
