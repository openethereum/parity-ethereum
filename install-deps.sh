#!/usr/bin/env bash

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
				[yY] ) echo; break ;;
				'' ) echo; break ;;
				[nN] ) abortInstall "${red}==>${reset} Process stopped by user. To resume the install run the one-liner command again." ;;
				* ) echo "Unrecognized option provided. Please provide either 'Y' or 'N'";
			esac
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
		ETH_PATH=`which eth 2>/dev/null`

		if [[ -f $ETH_PATH ]]
		then
			check "Found eth: $ETH_PATH"
			echo "$($ETH_PATH -V)"
			isEth=true
		else
			uncheck "Eth is missing"
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

		info "Updating brew"
		exe brew update
		echo

		info "Installing rocksdb"
		exe brew install rocksdb
		info "Installing multirust"
		exe brew install multirust
		sudo multirust update nightly
		sudo multirust default nightly
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
	function linux_rocksdb_installer()
	{
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
	}

	function linux_installer()
	{
		info "Installing git"
		sudo apt-get install -q -y git
		echo

		info "Installing rocksdb"
		linux_rocksdb_installer
		echo

		info "Installing multirust"
		curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sudo sh -s -- --yes
		sudo multirust update nightly
		sudo multirust default nightly
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
	}

	function verify_installation()
	{
		info "Verifying installation"
#		find_eth

#		if [[ $isEth == false ]]
#		then
#			abortInstall
#		fi
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
#		echo
#		successHeading "Installation successful!"
#		head "Next steps"
#		info "Run ${cyan}\`\`${reset} to get started.${reset}"
#		echo
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
	wait_for_user "${b}OK,${reset} let's go!"

	# Install dependencies and eth
	install

	# Check installation
	verify_installation

	# Display goodby message
	finish
}

run_installer