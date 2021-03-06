# By default, let's print out some help
.PHONY: usage
usage:
	@echo "$$(tput bold)Welcome to Tock for STM32!$$(tput sgr0)"
	@echo
	@echo "First things first, if you haven't yet, check out tock/doc/Getting_Started."
	@echo "You'll need to install a few requirements before we get going."
	@echo
	@echo "The next step is to choose a board to build Tock for."
	@echo "Tock for STM32 currently includes support for:"
	@ls -p boards/ | grep '/$$' | cut -d'/' -f1 | xargs echo "  "
	@echo
	@echo "Run 'make' in a board directory to build Tock for that board,"
	@echo "and usually 'make program' or 'make flash' to load Tock onto hardware."
	@echo "Check out the README in your board's folder for more information."
	@echo
	@echo "This root Makefile has a few useful targets as well:"
	@echo "  allboards: Compiles Tock for all supported boards"
	@echo "     alldoc: Builds Tock documentation for all boards"
	@echo "     format: Runs the rustfmt tool on all sources"
	@echo "       list: Lists available boards"
	@echo
	@echo "$$(tput bold)Happy Hacking!$$(tput sgr0)"

.PHONY: allboards
allboards:
	@for f in `./tock/tools/list_boards.sh -1`; do echo "$$(tput bold)Build $$f"; $(MAKE) -C "boards/$$f" || exit 1; done

.PHONY: alldoc
alldoc:
	@for f in `./tock/tools/list_boards.sh -1`; do echo "$$(tput bold)Documenting $$f"; $(MAKE) -C "boards/$$f" doc || exit 1; done

.PHONY: fmt format formatall
fmt format formatall:
	@./tools/run_cargo_fmt.sh

.PHONY: list list-boards list-platforms
list list-boards list-platforms:
	@./tock/tools/list_boards.sh
