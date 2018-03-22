## Platform specific instructions
The [SensorTag CC2650STK](http://www.ti.com/tool/CC2650STK) is a platform based on the CC2650 MCU by 
Texas Instrument, an SoC running on an ARM Cortex-M3. 

It comes with several peripherals attached:

* BLE Radio
* IR temperature sensor
* Humidity sensor
* Accelerometer
* Magnetometer
* Barometric pressure sensor
* On-Chip temperature sensor
* Battery/voltage sensor

You can read more about the sensors [here](http://processors.wiki.ti.com/index.php/SensorTag_User_Guide).


The technical reference manual for the cc2650 can be found 
[here](http://www.ti.com/lit/ug/swcu117h/swcu117h.pdf),
and it shares many properties with other MCUs in the same family (cc26xx).

### Flashing
Download and use [uniflash](http://processors.wiki.ti.com/index.php/Category:CCS_UniFlash) to flash. Follow the guide
[here](http://processors.wiki.ti.com/index.php/UniFlash_v4_Quick_Guide#Standalone_Command_line_tool) in order to generate
a standalone command line tool to ease the flashing process.

The standalone CLI has been extracted, set an environment variable named `UNIFLASH_CLI_BASE` in your shell profile:

```bash
$> echo export UNIFLASH_CLI_BASE="<path to extracted uniflash CLI>" >> ~/.bash_profile
$> source ~/.bash_profile
```

Now you're able to use the Makefile target `flash` in order to load the kernel onto the SensorTag board.

```bash
$> make flash       # make and flash the kernel
```

### Apps
To compile and get apps to the board, you need to navigate to their directory in tock/userland/...

```bash
$> cd tock/userland/examples/blink
$> make
```

#### Flashing
You can issue the flash command by pointing the variable `TOCK_BOARD` to the correct
directory.

```bash
$> make TOCK_BOARD=../../boards/sensortag flash 
```

### Debugging
You need to use openocd together with gdb in order to debug the launchxl board using JTAG. However, you'll need to build OpenOCD with extra applied patches until the next version has been released. 

Clone the repository and apply the patches:

```bash
$> git clone https://git.code.sf.net/p/openocd/code openocd 
$> cd openocd
$> git pull http://openocd.zylin.com/openocd refs/changes/22/4322/2 
$> git pull http://openocd.zylin.com/openocd refs/changes/58/4358/1
```

Once cloned and the patches has been applied, read the readme in order to build and install openocd.

Once flashed, launch openocd with the configuration specified at jtag/openocd.cfg:

```bash
$> openocd -f jtag/openocd.cfg
```

And then launch gdb

```bash
$> arm-none-eabi-gdb -x jtag/gdbinit
```

and it will automatically connect to the board.

*Note: it is possible to debug using OpenOCD without the applied patches, but it is painfully slow.
Just install a later version than 0.10 of OpenOCD, and uncomment the necessary parts in jtag/openocd.cfg (see file), then
follow the instructions above.*

### Panic/Crash
When the board panics or crashes, the RED led will be blinking frequently.