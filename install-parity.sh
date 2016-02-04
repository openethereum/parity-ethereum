#!/usr/bin/env bash


#TODO: replace with the main repo url
PARITY_DEB_URL=https://github.com/jesuscript/scripts/raw/master/parity.deb

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
  isDocker=false
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

  function get_osx_dependencies()
  {
    macos_version
    find_git
    find_ruby
    find_brew
  }

  function macos_version()
  {
    declare -a reqVersion
    declare -a localVersion

    depCount=$((depCount+1))
    OSX_VERSION=`/usr/bin/sw_vers -productVersion 2>/dev/null`

    if [ -z "$OSX_VERSION" ]
    then
      uncheck "OS X version not supported 🔥"
      isOsVersion=false
      canContinue=false
    else
      IFS='.' read -a localVersion <<< "$OSX_VERSION"
      IFS='.' read -a reqVersion <<< "$OSX_REQUIERED_VERSION"

      if (( ${reqVersion[0]} <= ${localVersion[0]} )) && (( ${reqVersion[1]} <= ${localVersion[1]} ))
      then
        check "OS X Version ${OSX_VERSION}"
        isOsVersion=true
        depFound=$((depFound+1))
        return
      else
        uncheck "OS X version not supported"
        isOsVersion=false
        canContinue=false
      fi
    fi

    errorMessages+="${red}==>${reset} ${b}Mac OS version too old:${reset} eth requires OS X version ${red}$OSX_REQUIERED_VERSION${reset} at least in order to run.\n"
    errorMessages+="    Please update the OS and reload the install process.\n"
  }

  function find_eth()
  {
    ETH_PATH=`which parity 2>/dev/null`

    if [[ -f $ETH_PATH ]]
    then
      check "Found parity: $ETH_PATH"
      isEth=true
    else
      uncheck "parity is missing"
      isEth=false
    fi
  }

  function find_git()
  {
    depCount=$((depCount+1))

    GIT_PATH=`which git 2>/dev/null`

    if [[ -f $GIT_PATH ]]
    then
      check "$($GIT_PATH --version)"
      isGit=true
      depFound=$((depFound+1))
    else
      uncheck "Git is missing"
      isGit=false
    fi
  }

  function find_ruby()
  {
    depCount=$((depCount+1))

    RUBY_PATH=`which ruby 2>/dev/null`

    if [[ -f $RUBY_PATH ]]
    then
      RUBY_VERSION=`ruby -e "print RUBY_VERSION"`
      check "Ruby ${RUBY_VERSION}"
      isRuby=true
      depFound=$((depFound+1))
    else
      uncheck "Ruby is missing 🔥"
      isRuby=false
      canContinue=false
      errorMessages+="${red}==>${reset} ${b}Couldn't find Ruby:${reset} Brew requires Ruby which could not be found.\n"
      errorMessages+="    Please install Ruby using these instructions ${u}${blue}https://www.ruby-lang.org/en/documentation/installation/${reset}.\n"
    fi
  }

  function find_brew()
  {
    BREW_PATH=`which brew 2>/dev/null`

    if [[ -f $BREW_PATH ]]
    then
      check "$($BREW_PATH -v)"
      isBrew=true
      depFound=$((depFound+1))
    else
      uncheck "Homebrew is missing"
      isBrew=false

      INSTALL_FILES+="${blue}${dim}==> Homebrew:${reset}\n"
      INSTALL_FILES+=" ${blue}${dim}➜${reset}  $HOMEBREW_PREFIX/bin/brew\n"
      INSTALL_FILES+=" ${blue}${dim}➜${reset}  $HOMEBREW_PREFIX/Library\n"
      INSTALL_FILES+=" ${blue}${dim}➜${reset}  $HOMEBREW_PREFIX/share/man/man1/brew.1\n"
    fi

    depCount=$((depCount+1))
  }

  function install_brew()
  {
    if [[ $isBrew == false ]]
    then
      head "Installing Homebrew"

      if [[ $isRuby == true ]]
      then
        ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
      else
        cd /usr

        if [[ ! -d $HOMEBREW_PREFIX ]]
        then
          sudo mkdir $HOMEBREW_PREFIX
          sudo chmod g+rwx $HOMEBREW_PREFIX
        fi

        if [[ ! -d $HOMEBREW_CACHE ]]
        then
          sudo mkdir $HOMEBREW_CACHE
          sudo chmod g+rwx $HOMEBREW_CACHE
        fi

        DEVELOPER_DIR=`/usr/bin/xcode-select -print-path 2>/dev/null`

        if [[ ! $(ls -A $DEVELOPER_DIR) || ! -f $DEVELOPER_DIR/usr/bin/git ]]
        then
          info "Installing the Command Line Tools (expect a GUI popup):"
          sudo /usr/bin/xcode-select --install

          echo "Press any key when the installation has completed"
        fi

        cd $HOMEBREW_PREFIX

        bash -o pipefail -c "curl -fsSL ${HOMEBREW_REPO}/tarball/master | tar xz -m --strip 1"
      fi

      find_brew
      echo

      if [[ $isBrew == false ]]
      then
        abortInstall "Couldn't install brew"
      fi
    fi
  }

  function osx_installer()
  {
    osx_dependency_installer

    info "Adding ethcore repository"
    brew tap ethcore/ethcore git@github.com:ethcore/homebrew-ethcore.git
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

  function osx_dependency_installer()
  {
    if [[ $isGit == false ]];
    then
      echo "Installing Git"
    fi

    if [[ $isRuby == false ]];
    then
      echo "Installing Ruby"
    fi

    if [[ $isBrew == false ]];
    then
      install_brew
    fi
  }

  function get_linux_dependencies()
  {
    find_apt
    find_docker
  }

  function find_apt()
  {
    APT_PATH=`which apt-get 2>/dev/null`

    if [[ -f $APT_PATH ]]
    then
      check "apt-get"
      echo "$($APT_PATH -v)"
      isApt=true
    else
      uncheck "apt-get is missing"
      isApt=false
    fi
  }

  function find_docker()
  {
    DOCKER_PATH=`which docker 2>/dev/null`

    if [[ -f $DOCKER_PATH ]]
    then
      check "docker"
      echo "$($DOCKER_PATH -v)"
      isDocker=true
    else
      isDocker=false
    fi
  }
  function linux_rocksdb_installer()
  {
    sudo add-apt-repository -y ppa:giskou/librocksdb
    sudo apt-get -f -y install
    sudo apt-get update
    sudo apt-get install -y librocksdb
  }

  function linux_installer()
  {
    info "Installing dependencies"
    sudo apt-get update && sudo apt-get install -q -y git curl g++ wget
    echo

    info "Installing rocksdb"
    linux_rocksdb_installer
    echo

    info "Installing parity"
    file=/tmp/parity.deb

    
    wget $PARITY_DEB_URL -qO $file
    sudo dpkg -i $file
    rm $file
  }

  function install_netstats()
  {
    echo "install netstats"

    if [[ $isDocker == false ]]
    then
      info "installing docker"
      curl -sSL https://get.docker.com/ | sh
    fi

    dir=$HOME/.netstats

    secret=$(prompt_for_input "Please enter the netstats secret:")
    instance_name=$(prompt_for_input "Please enter your instance name:")
    contact_details=$(prompt_for_input "Please enter your contact details (optional):")
    

    mkdir -p $dir
    cat > $dir/app.json << EOL
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

    sudo docker rm --force netstats-client 2> /dev/null
    sudo docker pull ethcore/netstats-client
    sudo docker run -d --net=host --name netstats-client -v $dir/app.json:/home/ethnetintel/eth-net-intelligence-api/app.json  ethcore/netstats-client 
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
  }

  function verify_installation()
  {
    info "Verifying installation"
    find_eth

    if [[ $isEth == false ]]
    then
      abortInstall
    fi
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
    successHeading "Installation successful!"
    #   head "Next steps"
    #   info "Run ${cyan}\`\`${reset} to get started.${reset}"
    echo
    exit 0
  }

  # Check dependencies
  head "Checking OS dependencies"
  detectOS

  echo
  head "In addition to the parity build dependencies, this script will install:"
  echo "$INSTALL_FILES"
  echo

  # Prompt user to continue or abort
  if wait_for_user "${b}OK,${reset} let's go!"
  then
    echo "Installing..."
  else
    abortInstall "${red}==>${reset} Process stopped by user. To resume the install run the one-liner command again."
  fi

  # Install dependencies and eth
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

  # Check installation
  verify_installation

  # Display goodby message
  finish
}

run_installer
