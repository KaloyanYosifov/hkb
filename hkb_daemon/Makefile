# Unix based system support only
DATA_DIR := $(HOME)/.local/share/hkb
CURRENT_OS := $(shell uname)

ifeq ($(CURRENT_OS),Darwin)
    DATA_DIR := $(HOME)/Library/Application\ Support/hkb
endif

sync_files_with_data_dir:
	mkdir -p $(DATA_DIR)/sound
	rsync -avP ./sounds/* $(DATA_DIR)/sound
