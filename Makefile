# Makefile for cross-compilation
IOS_ARCHS = i386-apple-ios x86_64-apple-ios armv7-apple-ios armv7s-apple-ios aarch64-apple-ios
IOS_LIB = libethcore_util.a

ios: $(IOS_LIB)

.PHONY: $(IOS_ARCHS)
$(IOS_ARCHS): %:
	multirust run ios cargo build --target $@

$(IOS_LIB): $(IOS_ARCHS)
	lipo -create -output $@ $(foreach arch,$(IOS_ARCHS),$(wildcard target/$(arch)/debug/$(IOS_LIB)))
