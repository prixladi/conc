default:

install: install_daemon install_cli install_gui

install_daemon: 
	cd apps/daemon && make install ; cd ../..

install_cli: 
	cd apps/cli && make install ; cd ../..
	
install_gui: 
	cd apps/gui && make install ; cd ../..