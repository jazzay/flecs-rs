run-examples:
	cargo run --example dynamic_components
	cargo run --example entity_basics
	cargo run --example entity_hierarchy
	cargo run --example entity_iterate_components
	cargo run --example prefabs
	cargo run --example relations
	cargo run --example filters
	cargo run --example hello_world
	cargo run --example systems
setup_emsdk:
	# Way to dangerous to automatically delete a directory, imagine if user set /
	@if [ -d $(EMSDK) ]; then echo "emsdk '$(EMSDK)' directory already exist, please delete-it manually"; exit 1; fi
	# Install EMSDK
	mkdir -p `dirname $(EMSDK)` && \
		cd `dirname $(EMSDK)` && \
		git clone https://github.com/emscripten-core/emsdk.git $(EMSDK) && \
		cd $(EMSDK) && \
		git checkout tags/3.1.10 && \
		echo "13e29bd55185e3c12802bc090b4507901856b2ba" > ./emscripten-releases-tot.txt && \
		./emsdk install tot && \
		./emsdk activate tot
	# Add to PATH
	source $(EMSDK)
	sudo echo "source $(EMSDK)" >> ~/.bashrc
	