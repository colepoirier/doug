# Doug

Doug is a WIP semi-automated to full manual VLSI Analog and Mixed Signal CAD design tool built with [Bevy](https://github.com/bevyengine/bevy) (an in-development rust game engine) and Dan Fritchman's excellent rust VLSI library [Layout21](https://github.com/dan-fritchman/Layout21).

### Usage instructions

Make sure you compile Doug in release mode by running `cargo r --release`, otherwise you will get ~10x worse performance and the app may be stuttery, freeze a lot, and be generally unpleasant to use.

When the app is finished compiling it will start. Select 'File->Load' and select one of the *.proto files from the 'libs/' directory in this repository. There are currently two GDS library files provided: dff1_lib.proto can be used without doing anything but only contains one cell; oscibear.proto contains dozens of digitial, analog, and mixed-signal designs, and hundreds of smaller cells which the larger designs use, in order to use it you will need to run `zstd -D oscibear_proto_ztd_dict oscibear.proto.zst` and then select the uncompresed proto file in the file picker dialog.

### Background on the ocscibear.proto library

'oscibear.proto' was compiled from a real [GDSII file](https://web.archive.org/web/20220321001443/https://github.com/ucberkeley-ee290c/OsciBear/blob/main/gds/user_analog_project_wrapper.gds.gz) that was produced by a UC Berkeley undergraduate Electrical Enginnering course in 2021. They used a fork of eFabless's [Caravel (Analog) User-Project Starter](https://web.archive.org/web/20220321001646/https://github.com/efabless/caravel_user_project_analog) to combine the many student designs - digitial, analog, and mixed signal - into as single System on a Chip (SOC). The [OsciBear SoC](https://github.com/ucberkeley-ee290c/OsciBear) was fabricated in 2021 on the SkyWater130nm node using a Multi-Project Wafer (MPW).

### Potential Bevy platform issues

You may run into to errors if Bevy's native linux dependencies are not already installed on your system. This can be easily resolved by following the instructions here <https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md>.

For now, I will only be focusing on testing on linux, but bevy has good cross-platform support so you should not have any issues running on Windows 10 or MacOS.
