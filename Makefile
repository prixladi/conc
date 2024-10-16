default:

install: install_daemon install_cli

install_daemon: 
	cd apps/daemon && make install ; cd ../..

install_cli: 
	cd apps/cli && make install ; cd ../..
	