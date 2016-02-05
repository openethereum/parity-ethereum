#!/usr/bin/env bash


GET_DEPS_URL=http://get-deps.ethcore.io

function run_installer()
{
	####### Init vars
	
	declare OS_TYPE

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
		echo "${green}${bold} ✓${reset}	 $1${reset}"
	}

	function uncheck() {
		echo "${red}${bold} ✘${reset}	 $1${reset}"
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

	function detectOS() {
		if [[ "$OSTYPE" == "linux-gnu" ]]
		then
			OS_TYPE="linux"
			linux_version
		elif [[ "$OSTYPE" == "darwin"* ]]
		then
			OS_TYPE="osx"
		else
			OS_TYPE="win"
			abortInstall "${red}==>${reset} ${b}OS not supported:${reset} parity one-liner currently support OS X and Linux.\nFor instructions on installing parity on other platforms please visit ${u}${blue}http://ethcore.io/${reset}"
		fi

		echo
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

	function build_parity()
	{
		oldpwd= $(pwd)
		info "Downloading Parity..."
		git clone git@github.com:ethcore/parity $HOME/parity
		cd $HOME/parity
		git submodule init
		git submodule update
		
		info "Building Parity..."
		cargo build --release

		sudo cp target/release/parity /usr/bin/

		cd $oldpwd

		echo
		info "Parity source code is in $(pwd)/parity"
		info "Run a client with: ${b}cargo run --release${reset} or just ${b}parity${reset}"
	}

	function linux_installer()
	{
		build_parity
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
		cd $oldpwd
	}
	

	function install()
	{
		if [[ $OS_TYPE == "osx" ]]
		then
			osx_installer
		elif [[ $OS_TYPE == "linux" ]]
		then
			linux_installer
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
		#		head "Next steps"
		#		info "Run ${cyan}\`\`${reset} to get started.${reset}"
		echo
		exit 0
	}

	bash <(curl $GET_DEPS_URL -L)

	detectOS

	# Prompt user to continue or abort
	if wait_for_user "${b}OK,${reset} let's install Parity now!"
	then
		echo "Installing..."
	else
		abortInstall "${red}==>${reset} Process stopped by user. To resume the install run the one-liner command again."
	fi

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
