# This Makefile is only used for packaging, and should not be used in any other
# context

.PHONY: all clean install

all:
	@# Do nothing

clean:
	@# Do nothing

install:
	mkdir -p $(DESTDIR)/usr/bin
	install -m 755 target/$(TARGET)/release/pihole_api $(DESTDIR)/usr/bin/pihole-API
