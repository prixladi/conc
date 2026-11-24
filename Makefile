default:

ifndef PREFIX
PREFIX = /usr/local/bin
endif
ifndef SYSTEMD_PREFIX
SYSTEMD_PREFIX = $(HOME)/.config/systemd/user
endif

export PREFIX
export SYSTEMD_PREFIX

install: install_daemon install_cli

install_daemon: 
	cd apps/daemon && make install && cd ../..

install_cli: 
	cd apps/cli && make install && cd ../..
	
install_gui: 
	cd apps/gui && make install && cd ../..

install_api: 
	cd apps/api && make install && cd ../..

install_vscode: 
	cd apps/vscode && make install && cd ../..