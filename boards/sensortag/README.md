## Platform specific instructions

### Flashing
Download and use [uniflash](http://processors.wiki.ti.com/index.php/Category:CCS_UniFlash) to flash. Follow the guide
[here](http://processors.wiki.ti.com/index.php/UniFlash_v4_Quick_Guide#Standalone_Command_line_tool) in order to generate
a standalone command line tool to ease the flashing process.

The standalone CLI has been extracted, set an environment variable named `UNIFLASH_CLI_BASE` in your shell profile:

```bash
$> echo UNIFLASH_CLI_BASE="<path to extracted uniflash CLI>" >> ~/.bash_profile
$> source ~/.bash_profile
```

Now you're able to use the Makefile targets `flash` and `flash-app` in order to load the program onto the SensorTag
board.

```bash
$> make flash       # make and flash the kernel
$> make flash-blink # make and flash the blink app
```

### Debugging
You need to use openocd together with gdb in order to debug the sensortag board using JTAG. Once flashed, simply launch openocd

```bash
$> openocd -f jtag/sensortag_openocd.cfg
```

And then launch gdb

```bash
$> arm-none-eabi-gdb -x jtag/gdbinit
```

and it will automatically connect to, and reset, the board.
