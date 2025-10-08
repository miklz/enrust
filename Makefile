# Default binary name
EXE ?= enrust

# Determine file extension
ifdef COMSPEC
    EXT := .exe
else
    EXT :=
endif

# Main target
$(EXE)$(EXT):
	cargo build --release
	cp target/release/enrust ./$(EXE)$(EXT)

clean:
	cargo clean
	rm -f $(EXE)$(EXT) $(EXE)-*

.PHONY: clean