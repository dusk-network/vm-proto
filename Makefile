SUBDIRS := $(wildcard ./contracts/*/.)

all: $(SUBDIRS)

test: $(SUBDIRS) ## Run the contracts' tests
	cargo test --all-features

$(SUBDIRS):
	$(MAKE) -C $@

.PHONY: help all test $(SUBDIRS)
