.PHONY: deps build package install

deps:
	npm ci

build: deps
	npm run compile

package: build
	npm run package

install: package
	./install.sh
