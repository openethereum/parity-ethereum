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
	n=$'\n'


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
		if [[ $( ask_user "$1" ) == false ]]; then
			abort_install "${red}==>${reset} Process stopped by user. To resume the install run the one-liner command again."
		fi
	}

	function ask_user() {
		while :
		do
			read -p "${blue}==>${reset} $1 [Y/n] " imp
			case $imp in
				[yY] ) echo true; break ;;
				'' ) echo true; break ;;
				[nN] ) echo false; break ;;
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

	function sudo() {
		if $isSudo; then
			`which sudo` "$@"
		else
			"$@"
		fi
	}

	function detect_os() {
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
			abort_install "${red}==>${reset} ${b}OS not supported:${reset} Parity one-liner currently support OS X and Linux.${n}For instructions on installing parity on other platforms please visit ${u}${blue}http://ethcore.io/${reset}"
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
				abort_install "$errorMessages";
			elif [[ $canContinue == false ]]
			then
				red "Some dependencies which cannot be auto-installed are missing ($depFound/$depCount)"
				abort_install "$errorMessages";
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

		errorMessages+="${red}==>${reset} ${b}Mac OS version too old:${reset} eth requires OS X version ${red}$OSX_REQUIERED_VERSION${reset} at least in order to run.${n}"
		errorMessages+="    Please update the OS and reload the install process.${n}"
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
			errorMessages+="${red}==>${reset} ${b}Couldn't find Ruby:${reset} Brew requires Ruby which could not be found.${n}"
			errorMessages+="    Please install Ruby using these instructions ${u}${blue}https://www.ruby-lang.org/en/documentation/installation/${reset}.${n}"
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

			INSTALL_FILES+="${blue}${dim}==> Homebrew:${reset}${n}"
			INSTALL_FILES+=" ${blue}${dim}➜${reset}  $HOMEBREW_PREFIX/bin/brew${n}"
			INSTALL_FILES+=" ${blue}${dim}➜${reset}  $HOMEBREW_PREFIX/Library${n}"
			INSTALL_FILES+=" ${blue}${dim}➜${reset}  $HOMEBREW_PREFIX/share/man/man1/brew.1${n}"
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
				abort_install "Couldn't install brew"
			fi
		fi
	}

	function osx_installer()
	{
		osx_dependency_installer

		info "Updating brew"
		exe brew update
		echo

		info "Installing multirust"
		exe brew install multirust
		sudo multirust default beta
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

	function linux_version()
	{
		source /etc/lsb-release
		
		if [[ $DISTRIB_ID == "Ubuntu" ]]; then
			if [[ $DISTRIB_RELEASE == "14.04" || $DISTRIB_RELEASE == "15.04" || $DISTRIB_RELEASE == "15.10" ]]; then
				check "Ubuntu"
				isUbuntu=true
			else
				check "Ubuntu, but version not supported"
				isUbuntu=false
			fi
		else
			check "Ubuntu not found"
			isUbuntu=false
		fi
	}

	function get_linux_dependencies()
	{
		linux_version

		find_multirust

		find_curl
		find_git
		find_make
		find_gcc

		find_apt
		find_sudo
	}

	function find_multirust()
	{
		depCount=$((depCount+2))
		MULTIRUST_PATH=`which multirust 2>/dev/null`
		if [[ -f $MULTIRUST_PATH ]]; then
			depFound=$((depFound+1))
			check "multirust"
			isMultirust=true
			if [[ $(multirust show-default 2>/dev/null | grep beta | wc -l) == 3 ]]; then
				depFound=$((depFound+1))
				check "rust beta"
				isMultirustBeta=true
			else
				uncheck "rust is not beta"
				isMultirustBeta=false
				INSTALL_FILES+="${blue}${dim}==> multirust -> rust beta:${reset}${n}"
			fi
		else
			uncheck "multirust is missing"
			uncheck "rust beta is missing"
			isMultirust=false
			isMultirustBeta=false
			INSTALL_FILES+="${blue}${dim}==> multirust:${reset}${n}"
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
				errorMessages+="${red}==>${reset} ${b}Couldn't find apt-get:${reset} We can only use apt-get in order to grab our dependencies.${n}"
				errorMessages+="    Please switch to a distribution such as Debian or Ubuntu or manually install the missing packages.${n}"
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
			INSTALL_FILES+="${blue}${dim}==> g++:${reset}${n}"
		fi
	}

	function find_sudo()
	{
		depCount=$((depCount+1))
		SUDO_PATH=`which sudo 2>/dev/null`

		if [[ -f $SUDO_PATH ]]
		then
			depFound=$((depFound+1))
			check "sudo"
			isSudo=true
		else
			uncheck "sudo is missing"
			if [[ `whoami` == "root" ]]; then
				if [[ $isApt == false && $isMultirust == false ]]; then
					canContinue=false
					errorMessages+="${red}==>${reset} ${b}Couldn't find sudo:${reset} Sudo is needed for the installation of multirust.${n}"
					errorMessages+="    Please ensure you have sudo installed or alternatively install multirust manually.${n}"
				fi

				isSudo=false
			else
				canContinue=false
				errorMessages+="${red}==>${reset} ${b}Couldn't find sudo:${reset} Root access is needed for parts of this installation.${n}"
				errorMessages+="    Please ensure you have sudo installed or alternatively run this script as root.${n}"
			fi
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
			INSTALL_FILES+="${blue}${dim}==> git:${reset}${n}"
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
			INSTALL_FILES+="${blue}${dim}==> make:${reset}${n}"
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
			INSTALL_FILES+="${blue}${dim}==> curl:${reset}${n}"
		fi
	}

	function linux_installer()
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

		if [[ $isMultirust == false ]]; then
			info "Installing multirust..."
			if [[ $isSudo == false ]]; then
				apt-get install -q -y sudo
			fi
			curl -sf https://raw.githubusercontent.com/brson/multirust/master/quick-install.sh | sudo sh -s -- --yes
			echo
		fi

		if [[ $isMultirustBeta == false ]]; then
			info "Installing rust beta..."
			multirust default beta
			echo
		fi
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

		if [[ $OS_TYPE == "linux" ]]; then
			find_curl
			find_git
			find_make
			find_gcc
			find_multirust

			if [[ $isCurl == false || $isGit == false || $isMake == false || $isGCC == false || $isMultirustBeta == false ]]; then
				abort_install
			fi
		fi
	}

	function build_parity()
	{
		info "Downloading Parity..."
		git clone https://github.com/ethcore/parity
		cd parity
		git submodule init
		git submodule update
		
		info "Building..."
		cargo build --release
		cd ..

		echo
		successHeading "Parity is built!"
		info "Parity source code is in ${b}$(pwd)/parity${reset}. From there, you can:"
		info "- Run a client & sync the chain with:"
		info "    ${b}cargo run --release${reset}"
		info "- Run a JSONRPC-capable client (for use with netstats) with:"
		info "    ${b}cargo run --release -- -j --jsonrpc-url 127.0.0.1:8545${reset}"
		info "- Run tests with:"
		info "    ${b}cargo test --release --features ethcore/json-tests -p ethcore${reset}"
		info "- Install the client with:"
		info "    ${b}sudo cp parity/target/release/parity${reset} /usr/local/bin"
		echo
	}

	function install_netstats()
	{
		echo "Installing netstats"

		secret=$(prompt_for_input "Please enter the netstats secret:")
		instance_name=$(prompt_for_input "Please enter your instance name:")
		contact_details=$(prompt_for_input "Please enter your contact details (optional):")
		
		# install ethereum & install dependencies
		sudo apt-get install -y -qq build-essential git unzip wget nodejs npm ntp cloud-utils

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

		[ ! -d "www" ] && git clone https://github.com/cubedro/eth-net-intelligence-api netstats
		cd netstats
		sudo npm install
		sudo npm install pm2 -g

		cat > app.json << EOL
[
	{
		"name"							: "node-app",
		"script"						: "app.js",
		"log_date_format"		: "YYYY-MM-DD HH:mm Z",
		"merge_logs"				: false,
		"watch"							: false,
		"max_restarts"			: 10,
		"exec_interpreter"	: "node",
		"exec_mode"					: "fork_mode",
		"env":
		{
			"NODE_ENV"				: "production",
			"RPC_HOST"				: "localhost",
			"RPC_PORT"				: "8545",
			"LISTENING_PORT"	: "30303",
			"INSTANCE_NAME"		: "${instance_name}",
			"CONTACT_DETAILS" : "${contact_details}",
			"WS_SERVER"				: "wss://rpc.ethstats.net",
			"WS_SECRET"				: "${secret}",
			"VERBOSITY"				: 2
		
		}
	}
]
EOL

		pm2 start app.json
		cd ..
	}

	function abort_install()
	{
		echo
		error "Installation aborted"
		echo -e "$1"
		echo
		exit 0
	}

	function finish()
	{
		echo
		successHeading "Installation successful!"
		echo
		exit 0
	}


	####### Run the script
	tput clear
	echo
	echo
	echo " ${blue}∷ ${b}${green} WELCOME TO PARITY ${reset} ${blue}∷${reset}"
	echo
	echo

	# Check dependencies
	head "Checking OS dependencies"
	detect_os

	if [[ $INSTALL_FILES != "" ]]; then
		echo
		head "In addition to the Parity build dependencies, this script will install:"
		echo "$INSTALL_FILES"
		echo
	fi

	# Prompt user to continue or abort
	wait_for_user "${b}Last chance!${reset} Sure you want to install this software?"

	# Install dependencies and eth
	install

	# Check installation
	verify_installation

	if [[ ! -e parity ]]; then
		# Maybe install parity
		if [[ $(ask_user "${b}Parity${reset} Would you like to download and build parity?") == true ]]; then
			# Do get parity.
			build_parity
		fi
	fi

	if [[ $OS_TYPE == "linux" && $DISTRIB_ID == "Ubuntu" ]]; then
		if [[ $(ask_user "${b}Netstats${reset} Would you like to download, install and configure a Netstats client?${n}${b}${red}WARNING: ${reset}${red}This will need a secret and reconfigure any existing node/NPM installation you have.${reset} ") == true ]]; then
			install_netstats
		fi
	fi

	# Display goodby message
	finish
}

run_installer
