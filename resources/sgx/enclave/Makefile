RA_TYPE ?= dcap
ARCH_LIBDIR ?= /lib/$(shell $(CC) -dumpmachine)

ifeq ($(DEBUG),1)
GRAMINE_LOG_LEVEL = debug
else
GRAMINE_LOG_LEVEL = error
endif

.PHONY: all
all: cosmian_vm.manifest.sgx cosmian_vm.sig

%.manifest: %.manifest.template
	gramine-manifest \
		-Dlog_level=$(GRAMINE_LOG_LEVEL) \
		-Ddomain_name=$(DOMAIN_NAME) \
		-Demail=$(EMAIL) \
		-Dra_type=$(RA_TYPE) \
		-Dra_client_spid=$(RA_CLIENT_SPID) \
		-Dra_client_linkable=$(RA_CLIENT_LINKABLE) \
		-Darch_libdir=$(ARCH_LIBDIR) \
		$< > $@

%.manifest.sgx %.sig: sgx-sign-%
	@:

.INTERMEDIATE: sgx-sign-%
sgx-sign-%: %.manifest
	gramine-sgx-sign \
		--manifest $< \
		--output $<.sgx

.PHONY: check
check: all
	@./run.sh

.PHONY: clean
clean:
	$(RM) -r \
		*.manifest \
		*.manifest.sgx \
		*.sig \
		*.token
